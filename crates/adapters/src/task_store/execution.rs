use super::*;

impl yonder_application::TaskExecutionStore for SqliteTaskStore {
    fn supports_pending_cancel(&self) -> bool {
        true
    }
    fn supports_execution_attempts(&self) -> bool {
        true
    }
    fn supports_browser_references(&self) -> bool {
        true
    }
    fn supports_controls(&self) -> bool {
        true
    }
    fn supports_wait_for_user(&self) -> bool {
        true
    }
    fn prepare_attempt(
        &mut self,
        requested: &ExecutionAttempt,
        expected: u64,
    ) -> Result<(Task, ExecutionAttempt), Error> {
        if expected == 0
            || expected >= i64::MAX as u64
            || requested.phase != AttemptPhase::Prepared
            || requested.accepted_sequence != 0
            || [
                requested.task_id.as_str(),
                requested.step_id.as_str(),
                requested.attempt_id.as_str(),
                requested.worker_instance_id.as_str(),
                requested.host_session_id.as_str(),
            ]
            .iter()
            .any(|id| !yonder_application::valid_id(id))
        {
            return Err(Error::InvalidInput);
        }
        let tx = self
            .0
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(storage)?;
        let row: Option<(String, i64, String, Option<String>, String)> = tx
            .query_row(
                "SELECT state,sequence,owner_agent_id,name,source FROM tasks WHERE id=?1",
                [&requested.task_id],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?)),
            )
            .optional()
            .map_err(storage)?;
        let (state, sequence, owner_agent_id, task_name, source) = row.ok_or(Error::NotFound)?;
        let task = Task {
            id: requested.task_id.clone(),
            owner_agent_id,
            name: task_name,
            source: task_source(&source)?,
            status: status(&state)?,
            sequence: u64::try_from(sequence).map_err(|_| Error::StorageUnavailable)?,
        };
        let existing: Option<(String,String,String,String,String,i64)> = tx.query_row("SELECT step_id,attempt_id,worker_instance_id,host_session_id,phase,accepted_sequence FROM task_attempts WHERE task_id=?1 AND attempt_id=?2", params![requested.task_id,requested.attempt_id], |r| Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?,r.get(4)?,r.get(5)?))).optional().map_err(storage)?;
        if let Some((step_id, attempt_id, worker_instance_id, host_session_id, phase, accepted)) =
            existing
        {
            let accepted_sequence =
                u64::try_from(accepted).map_err(|_| Error::StorageUnavailable)?;
            if phase != "prepared"
                || step_id != requested.step_id
                || worker_instance_id != requested.worker_instance_id
                || host_session_id != requested.host_session_id
                || accepted_sequence != expected + 1
            {
                return Err(Error::Conflict);
            }
            tx.commit().map_err(storage)?;
            return Ok((
                task,
                ExecutionAttempt {
                    task_id: requested.task_id.clone(),
                    step_id,
                    attempt_id,
                    worker_instance_id,
                    host_session_id,
                    phase: AttemptPhase::Prepared,
                    accepted_sequence,
                },
            ));
        }
        if !matches!(task.status, Status::Created | Status::Running) || task.sequence != expected {
            return Err(Error::Conflict);
        }
        let current_step: Option<(String,i64)> = tx.query_row("SELECT step_id,accepted_sequence FROM task_steps WHERE task_id=?1 ORDER BY accepted_sequence DESC LIMIT 1", [&requested.task_id], |r| Ok((r.get(0)?,r.get(1)?))).optional().map_err(storage)?;
        if current_step.as_ref().map(|value| value.0.as_str()) != Some(requested.step_id.as_str()) {
            return Err(Error::StopRequired);
        }
        if task.status == Status::Running {
            let previous: Option<(String,Option<String>,i64)> = tx.query_row("SELECT phase,control_id,COALESCE(stop_sequence,handback_sequence,0) FROM task_attempts WHERE task_id=?1 ORDER BY accepted_sequence DESC LIMIT 1",[&requested.task_id],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?))).optional().map_err(storage)?;
            let step_sequence = u64::try_from(current_step.as_ref().unwrap().1)
                .map_err(|_| Error::StorageUnavailable)?;
            if !matches!(previous,Some((ref phase,None,stop)) if matches!(phase.as_str(),"stopped"|"unknown") && stop>0 && step_sequence > u64::try_from(stop).unwrap_or(u64::MAX))
            {
                return Err(Error::StopRequired);
            }
        }
        Self::ensure_audit_capacity(&tx)?;
        let accepted = expected + 1;
        if tx.execute("UPDATE tasks SET state='running',sequence=?1 WHERE id=?2 AND state=?3 AND sequence=?4", params![accepted as i64,requested.task_id,state,expected as i64]).map_err(storage)? != 1 { return Err(Error::Conflict); }
        tx.execute(
            "INSERT INTO events(task_id,sequence,previous,state) VALUES (?1,?2,?3,'running')",
            params![requested.task_id, accepted as i64, state],
        )
        .map_err(storage)?;
        tx.execute(
            "INSERT INTO outbox(task_id,sequence) VALUES (?1,?2)",
            params![requested.task_id, accepted as i64],
        )
        .map_err(storage)?;
        tx.execute("INSERT INTO task_attempts(task_id,step_id,attempt_id,worker_instance_id,host_session_id,phase,accepted_sequence) VALUES (?1,?2,?3,?4,?5,'prepared',?6)", params![requested.task_id,requested.step_id,requested.attempt_id,requested.worker_instance_id,requested.host_session_id,accepted as i64]).map_err(storage)?;
        tx.commit().map_err(storage)?;
        let accepted_attempt = ExecutionAttempt {
            accepted_sequence: accepted,
            ..requested.clone()
        };
        Ok((
            Task {
                status: Status::Running,
                sequence: accepted,
                ..task
            },
            accepted_attempt,
        ))
    }

    fn get_attempt(&mut self, task_id: &str) -> Result<Option<ExecutionAttempt>, Error> {
        let row: Option<(String,String,String,String,String,i64,Option<i64>)> = self.0.query_row("SELECT step_id,attempt_id,worker_instance_id,host_session_id,phase,accepted_sequence,handback_sequence FROM task_attempts WHERE task_id=?1 ORDER BY accepted_sequence DESC LIMIT 1", [task_id], |r| Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?,r.get(4)?,r.get(5)?,r.get(6)?))).optional().map_err(storage)?;
        row.map(
            |(step_id, attempt_id, worker_instance_id, host_session_id, phase, accepted, handback)| {
                let phase = match phase.as_str() {
                    "prepared" => AttemptPhase::Prepared,
                    "observed" => AttemptPhase::Observed,
                    "unknown" if handback.is_some() => AttemptPhase::Stopped,
                    "unknown" => AttemptPhase::Unknown,
                    "stopped" => AttemptPhase::Stopped,
                    _ => return Err(Error::StorageUnavailable),
                };
                Ok(ExecutionAttempt {
                    task_id: task_id.into(),
                    step_id,
                    attempt_id,
                    worker_instance_id,
                    host_session_id,
                    phase,
                    accepted_sequence: u64::try_from(accepted)
                        .map_err(|_| Error::StorageUnavailable)?,
                })
            },
        )
        .transpose()
    }

    fn get_attempt_result(&mut self, task_id: &str) -> Result<Option<AttemptResultRecord>, Error> {
        let row:Option<(String,String,String,String,String,Option<i64>,Option<i64>,Option<String>,Option<i64>)>=self.0.query_row("SELECT step_id,attempt_id,worker_instance_id,host_session_id,phase,action_succeeded,observe_valid,unknown_reason,result_sequence FROM task_attempts WHERE task_id=?1 ORDER BY accepted_sequence DESC LIMIT 1",[task_id],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?,r.get(4)?,r.get(5)?,r.get(6)?,r.get(7)?,r.get(8)?))).optional().map_err(storage)?;
        row.map(
            |(
                step_id,
                attempt_id,
                worker_instance_id,
                host_session_id,
                phase,
                action,
                observe,
                reason_value,
                result,
            )| {
                let conclusion = match (phase.as_str(), action, observe, reason_value.as_deref()) {
                    ("observed" | "stopped", Some(value), Some(1), None) => {
                        AttemptConclusion::Observed {
                            action_succeeded: value != 0,
                        }
                    }
                    ("unknown", None, Some(0), Some(value)) => AttemptConclusion::Unknown {
                        reason: parse_reason(value)?,
                    },
                    _ => return Err(Error::StopRequired),
                };
                Ok(AttemptResultRecord {
                    task_id: task_id.into(),
                    step_id,
                    attempt_id,
                    worker_instance_id,
                    host_session_id,
                    conclusion,
                    result_sequence: u64::try_from(result.ok_or(Error::StorageUnavailable)?)
                        .map_err(|_| Error::StorageUnavailable)?,
                })
            },
        )
        .transpose()
    }

    fn record_attempt_result(
        &mut self,
        attempt: &ExecutionAttempt,
        expected: u64,
        conclusion: AttemptConclusion,
    ) -> Result<(Task, AttemptResultRecord), Error> {
        if expected == 0
            || expected >= i64::MAX as u64
            || attempt.accepted_sequence == 0
            || [
                attempt.task_id.as_str(),
                attempt.step_id.as_str(),
                attempt.attempt_id.as_str(),
                attempt.worker_instance_id.as_str(),
                attempt.host_session_id.as_str(),
            ]
            .iter()
            .any(|id| !yonder_application::valid_id(id))
        {
            return Err(Error::InvalidInput);
        }
        let tx = self
            .0
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(storage)?;
        let task_row: Option<(String, i64, String, Option<String>, String)> = tx
            .query_row(
                "SELECT state,sequence,owner_agent_id,name,source FROM tasks WHERE id=?1",
                [&attempt.task_id],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?)),
            )
            .optional()
            .map_err(storage)?;
        let (task_state, task_sequence, owner, name_value, source) =
            task_row.ok_or(Error::NotFound)?;
        let task = Task {
            id: attempt.task_id.clone(),
            owner_agent_id: owner,
            name: name_value,
            source: task_source(&source)?,
            status: status(&task_state)?,
            sequence: u64::try_from(task_sequence).map_err(|_| Error::StorageUnavailable)?,
        };
        if task.status != Status::Running || task.sequence != expected {
            return Err(Error::Conflict);
        }
        let row: Option<(String,String,String,String,String,i64,Option<i64>,Option<i64>,Option<i64>,Option<String>)> = tx.query_row(
            "SELECT step_id,attempt_id,worker_instance_id,host_session_id,phase,accepted_sequence,result_sequence,action_succeeded,observe_valid,unknown_reason FROM task_attempts WHERE task_id=?1 AND attempt_id=?2",
            params![attempt.task_id,attempt.attempt_id], |r| Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?,r.get(4)?,r.get(5)?,r.get(6)?,r.get(7)?,r.get(8)?,r.get(9)?)),
        ).optional().map_err(storage)?;
        let (
            step_id,
            attempt_id,
            worker_id,
            host_id,
            phase,
            accepted,
            result_sequence,
            action_succeeded,
            observe_valid,
            unknown_reason,
        ) = row.ok_or(Error::Conflict)?;
        if step_id != attempt.step_id
            || worker_id != attempt.worker_instance_id
            || host_id != attempt.host_session_id
            || u64::try_from(accepted).map_err(|_| Error::StorageUnavailable)?
                != attempt.accepted_sequence
        {
            return Err(Error::Conflict);
        }
        if phase != "prepared" {
            if phase == "stopped" {
                return Err(Error::Conflict);
            }
            let existing = match (
                phase.as_str(),
                action_succeeded,
                observe_valid,
                unknown_reason.as_deref(),
            ) {
                ("observed", Some(value), Some(1), None) => AttemptConclusion::Observed {
                    action_succeeded: value != 0,
                },
                ("unknown", None, Some(0), Some(value)) => AttemptConclusion::Unknown {
                    reason: parse_reason(value)?,
                },
                _ => return Err(Error::StorageUnavailable),
            };
            if existing != conclusion || result_sequence != Some(task_sequence) {
                return Err(Error::Conflict);
            }
            tx.commit().map_err(storage)?;
            return Ok((
                task,
                AttemptResultRecord {
                    task_id: attempt.task_id.clone(),
                    step_id,
                    attempt_id,
                    worker_instance_id: worker_id,
                    host_session_id: host_id,
                    conclusion: existing,
                    result_sequence: u64::try_from(task_sequence)
                        .map_err(|_| Error::StorageUnavailable)?,
                },
            ));
        }
        let sequence = expected + 1;
        let (phase_name, action_value, observe_value, reason_value) = match conclusion {
            AttemptConclusion::Observed { action_succeeded } => {
                ("observed", Some(i64::from(action_succeeded)), 1, None)
            }
            AttemptConclusion::Unknown { reason: unknown } => {
                ("unknown", None, 0, Some(reason(unknown)))
            }
        };
        if tx
            .execute(
                "UPDATE tasks SET sequence=?1 WHERE id=?2 AND state='running' AND sequence=?3",
                params![sequence as i64, attempt.task_id, expected as i64],
            )
            .map_err(storage)?
            != 1
        {
            return Err(Error::Conflict);
        }
        tx.execute("INSERT INTO events(task_id,sequence,previous,state) VALUES (?1,?2,'running','running')", params![attempt.task_id,sequence as i64]).map_err(storage)?;
        tx.execute(
            "INSERT INTO outbox(task_id,sequence) VALUES (?1,?2)",
            params![attempt.task_id, sequence as i64],
        )
        .map_err(storage)?;
        if tx.execute("UPDATE task_attempts SET phase=?1,result_sequence=?2,action_succeeded=?3,observe_valid=?4,unknown_reason=?5 WHERE task_id=?6 AND attempt_id=?7 AND phase='prepared'", params![phase_name,sequence as i64,action_value,observe_value,reason_value,attempt.task_id,attempt.attempt_id]).map_err(storage)? != 1 { return Err(Error::Conflict); }
        tx.commit().map_err(storage)?;
        Ok((
            Task { sequence, ..task },
            AttemptResultRecord {
                task_id: attempt.task_id.clone(),
                step_id,
                attempt_id,
                worker_instance_id: worker_id,
                host_session_id: host_id,
                conclusion,
                result_sequence: sequence,
            },
        ))
    }

    fn record_browser_result(
        &mut self,
        attempt: &ExecutionAttempt,
        expected: u64,
        reference: &BrowserReferenceRecord,
    ) -> Result<(Task, AttemptResultRecord), Error> {
        if expected == 0
            || expected >= i64::MAX as u64
            || attempt.accepted_sequence == 0
            || reference.task_id != attempt.task_id
            || !valid_ref(&reference.external_task_ref)
            || !matches!(
                reference.ownership.as_str(),
                "agent" | "agentDelegatedToUser" | "user"
            )
            || reference.managed_pages > 100
            || reference.updated_sequence != 0
        {
            return Err(Error::InvalidInput);
        }
        let tx = self
            .0
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(storage)?;
        let row: Option<(String, i64, String, Option<String>, String)> = tx
            .query_row(
                "SELECT state,sequence,owner_agent_id,name,source FROM tasks WHERE id=?1",
                [&attempt.task_id],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?)),
            )
            .optional()
            .map_err(storage)?;
        let (state, sequence, owner, name_value, source) = row.ok_or(Error::NotFound)?;
        let task = Task {
            id: attempt.task_id.clone(),
            owner_agent_id: owner,
            name: name_value,
            source: task_source(&source)?,
            status: status(&state)?,
            sequence: u64::try_from(sequence).map_err(|_| Error::StorageUnavailable)?,
        };
        if task.status != Status::Running || task.sequence != expected {
            return Err(Error::Conflict);
        }
        let current:Option<(String,String,String,String,String,i64,Option<i64>)>=tx.query_row("SELECT step_id,attempt_id,worker_instance_id,host_session_id,phase,accepted_sequence,result_sequence FROM task_attempts WHERE task_id=?1 AND attempt_id=?2",params![attempt.task_id,attempt.attempt_id],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?,r.get(4)?,r.get(5)?,r.get(6)?))).optional().map_err(storage)?;
        let (step_id, attempt_id, worker_id, host_id, phase, accepted, result_sequence) =
            current.ok_or(Error::Conflict)?;
        if step_id != attempt.step_id
            || worker_id != attempt.worker_instance_id
            || host_id != attempt.host_session_id
            || u64::try_from(accepted).map_err(|_| Error::StorageUnavailable)?
                != attempt.accepted_sequence
        {
            return Err(Error::Conflict);
        }
        if phase != "prepared" {
            if phase != "observed" || result_sequence != Some(sequence) {
                return Err(Error::Conflict);
            }
            let stored:Option<(String,String,i64,i64,i64)>=tx.query_row("SELECT external_task_ref,ownership,managed_pages,finished,updated_sequence FROM task_browser_refs WHERE task_id=?1",[&attempt.task_id],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?,r.get(4)?))).optional().map_err(storage)?;
            if !matches!(stored,Some((ref external,ref ownership,pages,finished,updated)) if external==&reference.external_task_ref && ownership==&reference.ownership && usize::try_from(pages).ok()==Some(reference.managed_pages) && (finished!=0)==reference.finished && updated==sequence)
            {
                return Err(Error::Conflict);
            }
            tx.commit().map_err(storage)?;
            return Ok((
                task,
                AttemptResultRecord {
                    task_id: attempt.task_id.clone(),
                    step_id,
                    attempt_id,
                    worker_instance_id: worker_id,
                    host_session_id: host_id,
                    conclusion: AttemptConclusion::Observed {
                        action_succeeded: true,
                    },
                    result_sequence: u64::try_from(sequence)
                        .map_err(|_| Error::StorageUnavailable)?,
                },
            ));
        }
        let next = expected + 1;
        if tx
            .execute(
                "UPDATE tasks SET sequence=?1 WHERE id=?2 AND state='running' AND sequence=?3",
                params![next as i64, attempt.task_id, expected as i64],
            )
            .map_err(storage)?
            != 1
        {
            return Err(Error::Conflict);
        }
        tx.execute("INSERT INTO events(task_id,sequence,previous,state) VALUES (?1,?2,'running','running')",params![attempt.task_id,next as i64]).map_err(storage)?;
        tx.execute(
            "INSERT INTO outbox(task_id,sequence) VALUES (?1,?2)",
            params![attempt.task_id, next as i64],
        )
        .map_err(storage)?;
        if tx.execute("UPDATE task_attempts SET phase='observed',result_sequence=?1,action_succeeded=1,observe_valid=1 WHERE task_id=?2 AND attempt_id=?3 AND phase='prepared'",params![next as i64,attempt.task_id,attempt.attempt_id]).map_err(storage)?!=1{return Err(Error::Conflict);}
        tx.execute("INSERT INTO task_browser_refs(task_id,external_task_ref,ownership,managed_pages,finished,updated_sequence) VALUES (?1,?2,?3,?4,?5,?6) ON CONFLICT(task_id) DO UPDATE SET external_task_ref=excluded.external_task_ref,ownership=excluded.ownership,managed_pages=excluded.managed_pages,finished=excluded.finished,updated_sequence=excluded.updated_sequence",params![attempt.task_id,reference.external_task_ref,reference.ownership,reference.managed_pages as i64,i64::from(reference.finished),next as i64]).map_err(storage)?;
        tx.commit().map_err(storage)?;
        Ok((
            Task {
                sequence: next,
                ..task
            },
            AttemptResultRecord {
                task_id: attempt.task_id.clone(),
                step_id,
                attempt_id,
                worker_instance_id: worker_id,
                host_session_id: host_id,
                conclusion: AttemptConclusion::Observed {
                    action_succeeded: true,
                },
                result_sequence: next,
            },
        ))
    }

    fn get_browser_reference(
        &mut self,
        task_id: &str,
    ) -> Result<Option<BrowserReferenceRecord>, Error> {
        if !yonder_application::valid_id(task_id) {
            return Err(Error::InvalidInput);
        }
        let row:Option<(String,String,i64,i64,i64)>=self.0.query_row("SELECT external_task_ref,ownership,managed_pages,finished,updated_sequence FROM task_browser_refs WHERE task_id=?1",[task_id],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?,r.get(4)?))).optional().map_err(storage)?;
        row.map(
            |(external_task_ref, ownership, managed_pages, finished, updated_sequence)| {
                Ok(BrowserReferenceRecord {
                    task_id: task_id.into(),
                    external_task_ref,
                    ownership,
                    managed_pages: usize::try_from(managed_pages)
                        .map_err(|_| Error::StorageUnavailable)?,
                    finished: finished != 0,
                    updated_sequence: u64::try_from(updated_sequence)
                        .map_err(|_| Error::StorageUnavailable)?,
                })
            },
        )
        .transpose()
    }

    fn advance_attempt(
        &mut self,
        attempt: &ExecutionAttempt,
        expected: u64,
    ) -> Result<(Task, StepBoundaryRecord), Error> {
        if expected == 0
            || expected >= i64::MAX as u64
            || !matches!(
                attempt.phase,
                AttemptPhase::Observed | AttemptPhase::Stopped
            )
        {
            return Err(Error::InvalidInput);
        }
        let tx = self
            .0
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(storage)?;
        let row: Option<(String, i64, String, Option<String>, String)> = tx
            .query_row(
                "SELECT state,sequence,owner_agent_id,name,source FROM tasks WHERE id=?1",
                [&attempt.task_id],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?)),
            )
            .optional()
            .map_err(storage)?;
        let (state, sequence, owner, name_value, source) = row.ok_or(Error::NotFound)?;
        let task = Task {
            id: attempt.task_id.clone(),
            owner_agent_id: owner,
            name: name_value,
            source: task_source(&source)?,
            status: status(&state)?,
            sequence: u64::try_from(sequence).map_err(|_| Error::StorageUnavailable)?,
        };
        if task.status != Status::Running || task.sequence != expected {
            return Err(Error::Conflict);
        }
        let current: Option<(String,String,String,String,String,i64,Option<i64>,Option<String>)> = tx.query_row("SELECT step_id,attempt_id,worker_instance_id,host_session_id,phase,accepted_sequence,stop_sequence,control_id FROM task_attempts WHERE task_id=?1 AND attempt_id=?2",params![attempt.task_id,attempt.attempt_id],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?,r.get(4)?,r.get(5)?,r.get(6)?,r.get(7)?))).optional().map_err(storage)?;
        let (step_id, attempt_id, worker_id, host_id, phase, accepted, stopped, control_id) =
            current.ok_or(Error::Conflict)?;
        if step_id != attempt.step_id
            || worker_id != attempt.worker_instance_id
            || host_id != attempt.host_session_id
            || u64::try_from(accepted).map_err(|_| Error::StorageUnavailable)?
                != attempt.accepted_sequence
        {
            return Err(Error::Conflict);
        }
        if phase == "stopped" {
            if control_id.is_some() {
                return Err(Error::StopRequired);
            }
            let stop_sequence = u64::try_from(stopped.ok_or(Error::StorageUnavailable)?)
                .map_err(|_| Error::StorageUnavailable)?;
            tx.commit().map_err(storage)?;
            return Ok((
                task,
                StepBoundaryRecord {
                    task_id: attempt.task_id.clone(),
                    step_id,
                    attempt_id,
                    worker_instance_id: worker_id,
                    host_session_id: host_id,
                    stop_sequence,
                },
            ));
        }
        if phase != "observed" {
            return Err(Error::StopRequired);
        }
        let pending: i64 = tx
            .query_row(
                "SELECT count(*) FROM task_controls WHERE task_id=?1 AND phase='pending'",
                [&attempt.task_id],
                |r| r.get(0),
            )
            .map_err(storage)?;
        if pending != 0 {
            return Err(Error::StopRequired);
        }
        let next = expected + 1;
        if tx
            .execute(
                "UPDATE tasks SET sequence=?1 WHERE id=?2 AND state='running' AND sequence=?3",
                params![next as i64, attempt.task_id, expected as i64],
            )
            .map_err(storage)?
            != 1
        {
            return Err(Error::Conflict);
        }
        tx.execute("INSERT INTO events(task_id,sequence,previous,state) VALUES (?1,?2,'running','running')",params![attempt.task_id,next as i64]).map_err(storage)?;
        tx.execute(
            "INSERT INTO outbox(task_id,sequence) VALUES (?1,?2)",
            params![attempt.task_id, next as i64],
        )
        .map_err(storage)?;
        if tx.execute("UPDATE task_attempts SET phase='stopped',stop_sequence=?1 WHERE task_id=?2 AND attempt_id=?3 AND phase='observed'",params![next as i64,attempt.task_id,attempt.attempt_id]).map_err(storage)?!=1{return Err(Error::Conflict);}
        tx.commit().map_err(storage)?;
        Ok((
            Task {
                sequence: next,
                ..task
            },
            StepBoundaryRecord {
                task_id: attempt.task_id.clone(),
                step_id,
                attempt_id,
                worker_instance_id: worker_id,
                host_session_id: host_id,
                stop_sequence: next,
            },
        ))
    }

    fn request_control(
        &mut self,
        attempt: &ExecutionAttempt,
        expected: u64,
        control_id: &str,
        kind: ControlKind,
    ) -> Result<(Task, ControlRequestRecord), Error> {
        if expected == 0 || expected >= i64::MAX as u64 || !yonder_application::valid_id(control_id)
        {
            return Err(Error::InvalidInput);
        }
        let kind_name = match kind {
            ControlKind::Pause => "pause",
            ControlKind::Cancel => "cancel",
            ControlKind::Takeover => "takeover",
        };
        let tx = self
            .0
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(storage)?;
        let row: Option<(String, i64, String, Option<String>, String)> = tx
            .query_row(
                "SELECT state,sequence,owner_agent_id,name,source FROM tasks WHERE id=?1",
                [&attempt.task_id],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?)),
            )
            .optional()
            .map_err(storage)?;
        let (state, sequence, owner, name_value, source) = row.ok_or(Error::NotFound)?;
        let task = Task {
            id: attempt.task_id.clone(),
            owner_agent_id: owner,
            name: name_value,
            source: task_source(&source)?,
            status: status(&state)?,
            sequence: u64::try_from(sequence).map_err(|_| Error::StorageUnavailable)?,
        };
        let existing: Option<(String,String,String,String,i64,Option<i64>,Option<String>,Option<String>)> = tx.query_row("SELECT attempt_id,control_id,kind,phase,accepted_sequence,stopped_sequence,focus_phase,focus_failure FROM task_controls WHERE task_id=?1", [&attempt.task_id], |r| Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?,r.get(4)?,r.get(5)?,r.get(6)?,r.get(7)?))).optional().map_err(storage)?;
        if let Some((
            attempt_id,
            stored_id,
            stored_kind,
            phase,
            accepted,
            stopped,
            focus,
            focus_error,
        )) = existing
        {
            let accepted = u64::try_from(accepted).map_err(|_| Error::StorageUnavailable)?;
            if attempt_id != attempt.attempt_id
                || stored_id != control_id
                || stored_kind != kind_name
                || (expected != accepted && expected.checked_add(1) != Some(accepted))
            {
                return Err(Error::Conflict);
            }
            let phase = match phase.as_str() {
                "pending" => ControlPhase::Pending,
                "stopped" => ControlPhase::Stopped,
                _ => return Err(Error::StorageUnavailable),
            };
            tx.commit().map_err(storage)?;
            let focus_phase = match focus.as_deref() {
                Some("locating") => Some(FocusPhase::Locating),
                Some("focused") => Some(FocusPhase::Focused),
                Some("failed") => Some(FocusPhase::Failed),
                None => None,
                _ => return Err(Error::StorageUnavailable),
            };
            let focus_failure = focus_error
                .as_deref()
                .map(parse_focus_failure)
                .transpose()?;
            return Ok((
                task,
                ControlRequestRecord {
                    task_id: attempt.task_id.clone(),
                    attempt_id,
                    control_id: stored_id,
                    kind,
                    phase,
                    accepted_sequence: accepted,
                    stopped_sequence: stopped
                        .map(u64::try_from)
                        .transpose()
                        .map_err(|_| Error::StorageUnavailable)?,
                    focus_phase,
                    focus_failure,
                },
            ));
        }
        if task.status != Status::Running || task.sequence != expected {
            return Err(Error::Conflict);
        }
        let accepted = expected + 1;
        let direct = attempt.phase == AttemptPhase::Stopped;
        let next_state = if direct {
            if kind == ControlKind::Cancel {
                "cancelled"
            } else {
                "paused"
            }
        } else {
            "running"
        };
        if direct {
            let uncontrolled:i64=tx.query_row("SELECT count(*) FROM task_attempts WHERE task_id=?1 AND attempt_id=?2 AND phase='stopped' AND control_id IS NULL",params![attempt.task_id,attempt.attempt_id],|r|r.get(0)).map_err(storage)?;
            if uncontrolled != 1 {
                return Err(Error::StopRequired);
            }
        }
        if tx.execute("UPDATE tasks SET state=?1,sequence=?2 WHERE id=?3 AND state='running' AND sequence=?4",params![next_state,accepted as i64,attempt.task_id,expected as i64]).map_err(storage)? != 1 { return Err(Error::Conflict); }
        tx.execute(
            "INSERT INTO events(task_id,sequence,previous,state) VALUES (?1,?2,'running',?3)",
            params![attempt.task_id, accepted as i64, next_state],
        )
        .map_err(storage)?;
        tx.execute(
            "INSERT INTO outbox(task_id,sequence) VALUES (?1,?2)",
            params![attempt.task_id, accepted as i64],
        )
        .map_err(storage)?;
        let control_phase = if direct { "stopped" } else { "pending" };
        tx.execute("INSERT INTO task_controls(task_id,attempt_id,control_id,kind,phase,accepted_sequence,stopped_sequence) VALUES (?1,?2,?3,?4,?5,?6,?7)",params![attempt.task_id,attempt.attempt_id,control_id,kind_name,control_phase,accepted as i64,if direct{Some(accepted as i64)}else{None}]).map_err(storage)?;
        if direct {
            tx.execute("UPDATE task_attempts SET control_id=?1,control_kind=?2 WHERE task_id=?3 AND attempt_id=?4 AND phase='stopped' AND control_id IS NULL",params![control_id,kind_name,attempt.task_id,attempt.attempt_id]).map_err(storage)?;
        }
        tx.commit().map_err(storage)?;
        Ok((
            Task {
                status: if direct {
                    if kind == ControlKind::Cancel {
                        Status::Cancelled
                    } else {
                        Status::Paused
                    }
                } else {
                    task.status
                },
                sequence: accepted,
                ..task
            },
            ControlRequestRecord {
                task_id: attempt.task_id.clone(),
                attempt_id: attempt.attempt_id.clone(),
                control_id: control_id.into(),
                kind,
                phase: if direct {
                    ControlPhase::Stopped
                } else {
                    ControlPhase::Pending
                },
                accepted_sequence: accepted,
                stopped_sequence: if direct { Some(accepted) } else { None },
                focus_phase: None,
                focus_failure: None,
            },
        ))
    }

    fn get_control(&mut self, task_id: &str) -> Result<Option<ControlRequestRecord>, Error> {
        let row: Option<(String,String,String,String,i64,Option<i64>,Option<String>,Option<String>)> = self.0.query_row("SELECT attempt_id,control_id,kind,phase,accepted_sequence,stopped_sequence,focus_phase,focus_failure FROM task_controls WHERE task_id=?1", [task_id], |r| Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?,r.get(4)?,r.get(5)?,r.get(6)?,r.get(7)?))).optional().map_err(storage)?;
        row.map(
            |(attempt_id, control_id, kind, phase, accepted, stopped, focus, focus_error)| {
                Ok(ControlRequestRecord {
                    task_id: task_id.into(),
                    attempt_id,
                    control_id,
                    kind: match kind.as_str() {
                        "pause" => ControlKind::Pause,
                        "cancel" => ControlKind::Cancel,
                        "takeover" => ControlKind::Takeover,
                        _ => return Err(Error::StorageUnavailable),
                    },
                    phase: match phase.as_str() {
                        "pending" => ControlPhase::Pending,
                        "stopped" => ControlPhase::Stopped,
                        _ => return Err(Error::StorageUnavailable),
                    },
                    accepted_sequence: u64::try_from(accepted)
                        .map_err(|_| Error::StorageUnavailable)?,
                    stopped_sequence: stopped
                        .map(u64::try_from)
                        .transpose()
                        .map_err(|_| Error::StorageUnavailable)?,
                    focus_phase: match focus.as_deref() {
                        Some("locating") => Some(FocusPhase::Locating),
                        Some("focused") => Some(FocusPhase::Focused),
                        Some("failed") => Some(FocusPhase::Failed),
                        None => None,
                        _ => return Err(Error::StorageUnavailable),
                    },
                    focus_failure: focus_error
                        .as_deref()
                        .map(parse_focus_failure)
                        .transpose()?,
                })
            },
        )
        .transpose()
    }

    fn begin_focus(
        &mut self,
        task_id: &str,
        control_id: &str,
    ) -> Result<(Task, ControlRequestRecord), Error> {
        self.update_focus(task_id, control_id, None)
    }
    fn finish_focus(
        &mut self,
        task_id: &str,
        control_id: &str,
        failure: Option<FocusFailure>,
    ) -> Result<(Task, ControlRequestRecord), Error> {
        self.update_focus(task_id, control_id, Some(failure))
    }

    fn stop_attempt(
        &mut self,
        attempt: &ExecutionAttempt,
        expected: u64,
        control_id: &str,
        kind: ControlKind,
    ) -> Result<(Task, StopRecord), Error> {
        if expected == 0
            || expected >= i64::MAX as u64
            || !yonder_application::valid_id(control_id)
            || attempt.accepted_sequence == 0
            || [
                attempt.task_id.as_str(),
                attempt.step_id.as_str(),
                attempt.attempt_id.as_str(),
                attempt.worker_instance_id.as_str(),
                attempt.host_session_id.as_str(),
            ]
            .iter()
            .any(|id| !yonder_application::valid_id(id))
        {
            return Err(Error::InvalidInput);
        }
        let kind_name = match kind {
            ControlKind::Pause => "pause",
            ControlKind::Cancel => "cancel",
            ControlKind::Takeover => "takeover",
        };
        let action = match kind {
            ControlKind::Pause | ControlKind::Takeover => Action::Pause,
            ControlKind::Cancel => Action::Cancel,
        };
        let tx = self
            .0
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(storage)?;
        let task_row: Option<(String, i64, String, Option<String>, String)> = tx
            .query_row(
                "SELECT state,sequence,owner_agent_id,name,source FROM tasks WHERE id=?1",
                [&attempt.task_id],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?)),
            )
            .optional()
            .map_err(storage)?;
        let (task_state, task_sequence, owner, name_value, source) =
            task_row.ok_or(Error::NotFound)?;
        let task = Task {
            id: attempt.task_id.clone(),
            owner_agent_id: owner,
            name: name_value,
            source: task_source(&source)?,
            status: status(&task_state)?,
            sequence: u64::try_from(task_sequence).map_err(|_| Error::StorageUnavailable)?,
        };
        if task.sequence != expected {
            return Err(Error::Conflict);
        }
        let control: Option<(String,String,String)> = tx.query_row("SELECT attempt_id,kind,phase FROM task_controls WHERE task_id=?1 AND control_id=?2",params![attempt.task_id,control_id],|r| Ok((r.get(0)?,r.get(1)?,r.get(2)?))).optional().map_err(storage)?;
        if control
            .as_ref()
            .is_none_or(|(attempt_id, stored_kind, phase)| {
                attempt_id != &attempt.attempt_id
                    || stored_kind != kind_name
                    || !matches!(phase.as_str(), "pending" | "stopped")
            })
        {
            return Err(Error::StopRequired);
        }
        let row: Option<(String,String,String,String,String,i64,Option<i64>,Option<i64>,Option<String>,Option<String>)> = tx.query_row(
            "SELECT step_id,attempt_id,worker_instance_id,host_session_id,phase,accepted_sequence,stop_sequence,action_succeeded,control_id,control_kind FROM task_attempts WHERE task_id=?1 AND attempt_id=?2",
            params![attempt.task_id,attempt.attempt_id], |r| Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?,r.get(4)?,r.get(5)?,r.get(6)?,r.get(7)?,r.get(8)?,r.get(9)?)),
        ).optional().map_err(storage)?;
        let (
            step_id,
            attempt_id,
            worker_id,
            host_id,
            phase,
            accepted,
            stop_sequence,
            action_succeeded,
            stored_control,
            stored_kind,
        ) = row.ok_or(Error::Conflict)?;
        if step_id != attempt.step_id
            || worker_id != attempt.worker_instance_id
            || host_id != attempt.host_session_id
            || u64::try_from(accepted).map_err(|_| Error::StorageUnavailable)?
                != attempt.accepted_sequence
        {
            return Err(Error::Conflict);
        }
        if phase == "stopped" {
            let stop_sequence = stop_sequence.ok_or(Error::StorageUnavailable)?;
            let expected_status = if kind == ControlKind::Cancel {
                Status::Cancelled
            } else {
                Status::Paused
            };
            if stored_control.as_deref() != Some(control_id)
                || stored_kind.as_deref() != Some(kind_name)
                || stop_sequence != task_sequence
                || task.status != expected_status
                || control
                    .as_ref()
                    .is_none_or(|(_, _, phase)| phase != "stopped")
            {
                return Err(Error::Conflict);
            }
            tx.commit().map_err(storage)?;
            return Ok((
                task,
                StopRecord {
                    task_id: attempt.task_id.clone(),
                    step_id,
                    attempt_id,
                    worker_instance_id: worker_id,
                    host_session_id: host_id,
                    control_id: control_id.into(),
                    kind,
                    stop_sequence: u64::try_from(stop_sequence)
                        .map_err(|_| Error::StorageUnavailable)?,
                },
            ));
        }
        if phase != "observed" || action_succeeded.is_none() || task.status != Status::Running {
            return Err(Error::StopRequired);
        }
        let transition = task
            .status
            .transition(expected, action)
            .map_err(Error::InvalidTransition)?;
        if tx.execute("UPDATE tasks SET state=?1,sequence=?2 WHERE id=?3 AND state='running' AND sequence=?4",params![name(transition.next),transition.sequence as i64,attempt.task_id,expected as i64]).map_err(storage)? != 1 { return Err(Error::Conflict); }
        tx.execute(
            "INSERT INTO events(task_id,sequence,previous,state) VALUES (?1,?2,'running',?3)",
            params![
                attempt.task_id,
                transition.sequence as i64,
                name(transition.next)
            ],
        )
        .map_err(storage)?;
        tx.execute(
            "INSERT INTO outbox(task_id,sequence) VALUES (?1,?2)",
            params![attempt.task_id, transition.sequence as i64],
        )
        .map_err(storage)?;
        if tx.execute("UPDATE task_attempts SET phase='stopped',stop_sequence=?1,control_id=?2,control_kind=?3 WHERE task_id=?4 AND attempt_id=?5 AND phase='observed'",params![transition.sequence as i64,control_id,kind_name,attempt.task_id,attempt.attempt_id]).map_err(storage)? != 1 { return Err(Error::Conflict); }
        if tx.execute("UPDATE task_controls SET phase='stopped',stopped_sequence=?1 WHERE task_id=?2 AND control_id=?3 AND phase='pending'",params![transition.sequence as i64,attempt.task_id,control_id]).map_err(storage)? != 1 { return Err(Error::Conflict); }
        tx.commit().map_err(storage)?;
        Ok((
            Task {
                status: transition.next,
                sequence: transition.sequence,
                ..task
            },
            StopRecord {
                task_id: attempt.task_id.clone(),
                step_id,
                attempt_id,
                worker_instance_id: worker_id,
                host_session_id: host_id,
                control_id: control_id.into(),
                kind,
                stop_sequence: transition.sequence,
            },
        ))
    }

    fn record_observation(
        &mut self,
        task_id: &str,
        attempt_id: &str,
        expected: u64,
        result: TaskObservationResult,
        summary: &str,
    ) -> Result<Task, Error> {
        if !yonder_application::valid_id(task_id)
            || !yonder_application::valid_id(attempt_id)
            || expected == 0
            || expected >= i64::MAX as u64
            || summary.trim() != summary
            || summary.is_empty()
            || summary.as_bytes().len() > 2048
            || summary
                .chars()
                .any(|value| value.is_control() && value != '\n')
        {
            return Err(Error::InvalidInput);
        }
        let tx = self
            .0
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(storage)?;
        let row: Option<(String, i64, String, Option<String>, String)> = tx
            .query_row(
                "SELECT state,sequence,owner_agent_id,name,source FROM tasks WHERE id=?1",
                [task_id],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?)),
            )
            .optional()
            .map_err(storage)?;
        let (state, sequence, owner_agent_id, name, source) = row.ok_or(Error::NotFound)?;
        let task = Task {
            id: task_id.into(),
            owner_agent_id,
            name,
            source: task_source(&source)?,
            status: status(&state)?,
            sequence: u64::try_from(sequence).map_err(|_| Error::StorageUnavailable)?,
        };
        if task.status != Status::Running || task.sequence != expected {
            return Err(Error::Conflict);
        }
        let attempt: Option<(String,String,Option<i64>,Option<String>)> = tx.query_row(
            "SELECT step_id,phase,action_succeeded,unknown_reason FROM task_attempts WHERE task_id=?1 AND attempt_id=?2 AND accepted_sequence=(SELECT max(accepted_sequence) FROM task_attempts WHERE task_id=?1)",
            params![task_id,attempt_id], |r| Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?)),
        ).optional().map_err(storage)?;
        let (step_id, phase, action_succeeded, unknown_reason) = attempt.ok_or(Error::Conflict)?;
        if matches!(phase.as_str(), "observed") && action_succeeded.is_none()
            || matches!(phase.as_str(), "unknown")
                && (result != TaskObservationResult::Unknown || unknown_reason.is_none())
            || !matches!(phase.as_str(), "observed" | "unknown")
        {
            return Err(Error::Conflict);
        }
        let next = expected + 1;
        if tx.execute("UPDATE tasks SET sequence=?1,observation_step_id=?2,observation_result=?3,observation_summary=?4 WHERE id=?5 AND state='running' AND sequence=?6", params![next as i64,step_id,observation_result_name(result),summary,task_id,expected as i64]).map_err(storage)? != 1 { return Err(Error::Conflict); }
        tx.execute("INSERT INTO events(task_id,sequence,previous,state) VALUES (?1,?2,'running','running')", params![task_id,next as i64]).map_err(storage)?;
        tx.execute(
            "INSERT INTO outbox(task_id,sequence) VALUES (?1,?2)",
            params![task_id, next as i64],
        )
        .map_err(storage)?;
        let payload = format!(
            r#"{{"step_id":"{step_id}","result":"{}","summary":{}}}"#,
            observation_result_name(result),
            serde_json::to_string(summary).map_err(|_| Error::StorageUnavailable)?
        );
        tx.execute("INSERT INTO task_presentation_events(task_id,sequence,kind,payload) VALUES (?1,?2,'observation',?3)", params![task_id,next as i64,payload]).map_err(storage)?;
        tx.commit().map_err(storage)?;
        Ok(Task {
            sequence: next,
            ..task
        })
    }

    fn declare_next_intent(
        &mut self,
        owner: Option<&str>,
        task_id: &str,
        expected: u64,
        intent: &str,
    ) -> Result<Task, Error> {
        if !yonder_application::valid_id(task_id)
            || expected == 0
            || expected >= i64::MAX as u64
            || intent.trim() != intent
            || intent.is_empty()
            || intent.as_bytes().len() > 1024
            || intent
                .chars()
                .any(|value| value.is_control() && value != '\n')
        {
            return Err(Error::InvalidInput);
        }
        let tx = self
            .0
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(storage)?;
        let row: Option<(String, i64, String, Option<String>, String)> = tx
            .query_row(
                "SELECT state,sequence,owner_agent_id,name,source FROM tasks WHERE id=?1",
                [task_id],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?)),
            )
            .optional()
            .map_err(storage)?;
        let (state, sequence, owner_agent_id, name, source) = row.ok_or(Error::NotFound)?;
        if owner.is_some_and(|owner| owner != owner_agent_id) {
            return Err(Error::NotFound);
        }
        let task = Task {
            id: task_id.into(),
            owner_agent_id,
            name,
            source: task_source(&source)?,
            status: status(&state)?,
            sequence: u64::try_from(sequence).map_err(|_| Error::StorageUnavailable)?,
        };
        if task.sequence != expected
            || matches!(
                task.status,
                Status::Completed | Status::Failed | Status::Cancelled
            )
        {
            return Err(Error::Conflict);
        }
        let next = expected + 1;
        if tx.execute("UPDATE tasks SET sequence=?1,next_intent=?2 WHERE id=?3 AND sequence=?4 AND state NOT IN ('completed','failed','cancelled')", params![next as i64,intent,task_id,expected as i64]).map_err(storage)? != 1 { return Err(Error::Conflict); }
        tx.execute(
            "INSERT INTO events(task_id,sequence,previous,state) VALUES (?1,?2,?3,?3)",
            params![task_id, next as i64, state],
        )
        .map_err(storage)?;
        tx.execute(
            "INSERT INTO outbox(task_id,sequence) VALUES (?1,?2)",
            params![task_id, next as i64],
        )
        .map_err(storage)?;
        tx.execute("INSERT INTO task_presentation_events(task_id,sequence,kind,payload) VALUES (?1,?2,'next-intent',?3)", params![task_id,next as i64,format!(r#"{{"next_intent":{}}}"#, serde_json::to_string(intent).map_err(|_| Error::StorageUnavailable)?) ]).map_err(storage)?;
        tx.commit().map_err(storage)?;
        Ok(Task {
            sequence: next,
            ..task
        })
    }

    fn wait_for_user(&mut self, task_id: &str, expected: u64, reason: &str) -> Result<Task, Error> {
        if !yonder_application::valid_id(task_id)
            || expected == 0
            || expected >= i64::MAX as u64
            || reason.trim() != reason
            || reason.is_empty()
            || reason.as_bytes().len() > 512
            || reason.chars().any(char::is_control)
        {
            return Err(Error::InvalidInput);
        }
        let tx = self
            .0
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(storage)?;
        let row: Option<(String, i64, String, Option<String>, String)> = tx
            .query_row(
                "SELECT state,sequence,owner_agent_id,name,source FROM tasks WHERE id=?1",
                [task_id],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?)),
            )
            .optional()
            .map_err(storage)?;
        let (state, sequence, owner_agent_id, task_name, source) = row.ok_or(Error::NotFound)?;
        if state != "running" || sequence != expected as i64 {
            return Err(Error::Conflict);
        }
        let pending: i64 = tx
            .query_row(
                "SELECT count(*) FROM task_controls WHERE task_id=?1 AND phase='pending'",
                [task_id],
                |r| r.get(0),
            )
            .map_err(storage)?;
        let phase:Option<String>=tx.query_row("SELECT phase FROM task_attempts WHERE task_id=?1 ORDER BY accepted_sequence DESC LIMIT 1",[task_id],|r|r.get(0)).optional().map_err(storage)?;
        if pending != 0 || phase.as_deref() != Some("stopped") {
            return Err(Error::StopRequired);
        }
        let next = expected + 1;
        if tx.execute("UPDATE tasks SET state='waiting-for-user',sequence=?1 WHERE id=?2 AND state='running' AND sequence=?3",params![next as i64,task_id,expected as i64]).map_err(storage)?!=1{return Err(Error::Conflict);}
        tx.execute("INSERT INTO events(task_id,sequence,previous,state,wait_reason) VALUES (?1,?2,'running','waiting-for-user',?3)",params![task_id,next as i64,reason]).map_err(storage)?;
        tx.execute(
            "INSERT INTO outbox(task_id,sequence) VALUES (?1,?2)",
            params![task_id, next as i64],
        )
        .map_err(storage)?;
        tx.commit().map_err(storage)?;
        Ok(Task {
            id: task_id.into(),
            owner_agent_id,
            name: task_name,
            source: task_source(&source)?,
            status: Status::WaitingForUser,
            sequence: next,
        })
    }

}
