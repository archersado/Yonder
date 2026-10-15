use super::*;

impl yonder_application::TaskStore for SqliteTaskStore {
}

impl yonder_application::TaskCoreStore for SqliteTaskStore {
    fn declare_step(
        &mut self,
        owner: &str,
        task_id: &str,
        expected: u64,
        step_id: &str,
        label: &str,
    ) -> Result<(Task, StepDeclaration), Error> {
        if [owner, task_id, step_id]
            .iter()
            .any(|id| !yonder_application::valid_id(id))
            || expected == 0
            || expected >= i64::MAX as u64
            || !yonder_application::valid_step_label(label)
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
        let (state, sequence, actual_owner, task_name, source) = row.ok_or(Error::NotFound)?;
        if actual_owner != owner {
            return Err(Error::NotFound);
        }
        let current = Task {
            id: task_id.into(),
            owner_agent_id: actual_owner,
            name: task_name,
            source: task_source(&source)?,
            status: status(&state)?,
            sequence: u64::try_from(sequence).map_err(|_| Error::StorageUnavailable)?,
        };
        let existing: Option<(String, i64)> = tx
            .query_row(
                "SELECT label,accepted_sequence FROM task_steps WHERE task_id=?1 AND step_id=?2",
                params![task_id, step_id],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .optional()
            .map_err(storage)?;
        if let Some((original, accepted)) = existing {
            if original != label {
                return Err(Error::StepConflict);
            }
            tx.commit().map_err(storage)?;
            return Ok((
                current,
                StepDeclaration {
                    step_id: step_id.into(),
                    label: original,
                    accepted_sequence: u64::try_from(accepted)
                        .map_err(|_| Error::StorageUnavailable)?,
                },
            ));
        }
        if !matches!(current.status, Status::Created | Status::Running) {
            return Err(Error::StopRequired);
        }
        if current.sequence != expected {
            return Err(Error::Conflict);
        }
        if current.status == Status::Running {
            let boundary: Option<(String,Option<String>,Option<i64>)> = tx.query_row("SELECT phase,control_id,handback_sequence FROM task_attempts WHERE task_id=?1 ORDER BY accepted_sequence DESC LIMIT 1",[task_id],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?))).optional().map_err(storage)?;
            let pending: i64 = tx
                .query_row(
                    "SELECT count(*) FROM task_controls WHERE task_id=?1 AND phase='pending'",
                    [task_id],
                    |r| r.get(0),
                )
                .map_err(storage)?;
            if !matches!(boundary,Some((ref phase,None,handback)) if phase=="stopped" || phase=="unknown" && handback.is_some()) || pending != 0 {
                return Err(Error::StopRequired);
            }
        }
        let task_count: i64 = tx
            .query_row(
                "SELECT count(*) FROM task_steps WHERE task_id=?1",
                [task_id],
                |r| r.get(0),
            )
            .map_err(storage)?;
        let total_count: i64 = tx
            .query_row("SELECT count(*) FROM task_steps", [], |r| r.get(0))
            .map_err(storage)?;
        if task_count >= 1024 || total_count >= 10000 {
            return Err(Error::QuotaExceeded);
        }
        let accepted = expected + 1;
        if tx.execute("UPDATE tasks SET sequence=?1 WHERE id=?2 AND owner_agent_id=?3 AND state=?4 AND sequence=?5", params![accepted as i64,task_id,owner,state,expected as i64]).map_err(storage)? != 1 { return Err(Error::Conflict); }
        tx.execute(
            "INSERT INTO events(task_id,sequence,previous,state) VALUES (?1,?2,?3,?3)",
            params![task_id, accepted as i64, state],
        )
        .map_err(storage)?;
        tx.execute(
            "INSERT INTO outbox(task_id,sequence) VALUES (?1,?2)",
            params![task_id, accepted as i64],
        )
        .map_err(storage)?;
        tx.execute(
            "INSERT INTO task_steps(task_id,step_id,label,accepted_sequence) VALUES (?1,?2,?3,?4)",
            params![task_id, step_id, label, accepted as i64],
        )
        .map_err(storage)?;
        tx.commit().map_err(storage)?;
        Ok((
            Task {
                sequence: accepted,
                ..current
            },
            StepDeclaration {
                step_id: step_id.into(),
                label: label.into(),
                accepted_sequence: accepted,
            },
        ))
    }

    fn get_with_step(&mut self, id: &str) -> Result<(Task, Option<StepDeclaration>), Error> {
        let tx = self.0.transaction().map_err(storage)?;
        let row: Option<(String, i64, String, Option<String>, String)> = tx
            .query_row(
                "SELECT state,sequence,owner_agent_id,name,source FROM tasks WHERE id=?1",
                [id],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?)),
            )
            .optional()
            .map_err(storage)?;
        let (state, sequence, owner_agent_id, name, source) = row.ok_or(Error::NotFound)?;
        let step: Option<(String,String,i64)> = tx.query_row("SELECT step_id,label,accepted_sequence FROM task_steps WHERE task_id=?1 ORDER BY accepted_sequence DESC LIMIT 1", [id], |r| Ok((r.get(0)?,r.get(1)?,r.get(2)?))).optional().map_err(storage)?;
        tx.commit().map_err(storage)?;
        let step = match step {
            Some((step_id, label, accepted)) => Some(StepDeclaration {
                step_id,
                label,
                accepted_sequence: u64::try_from(accepted)
                    .map_err(|_| Error::StorageUnavailable)?,
            }),
            None => None,
        };
        Ok((
            Task {
                id: id.into(),
                owner_agent_id,
                name,
                source: task_source(&source)?,
                status: status(&state)?,
                sequence: u64::try_from(sequence).map_err(|_| Error::StorageUnavailable)?,
            },
            step,
        ))
    }

    fn get_presentation(&mut self, id: &str) -> Result<(Task, TaskPresentation), Error> {
        let tx = self.0.transaction().map_err(storage)?;
        let row: Option<(String,i64,String,Option<String>,String,Option<String>,Option<String>,Option<String>,Option<String>)> = tx.query_row(
            "SELECT state,sequence,owner_agent_id,name,source,observation_step_id,observation_result,observation_summary,next_intent FROM tasks WHERE id=?1", [id],
            |r| Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?,r.get(4)?,r.get(5)?,r.get(6)?,r.get(7)?,r.get(8)?)),
        ).optional().map_err(storage)?;
        let (
            state,
            sequence,
            owner_agent_id,
            name,
            source,
            observation_step_id,
            observation_result_value,
            observation_summary,
            next_intent,
        ) = row.ok_or(Error::NotFound)?;
        let step: Option<(String,String,i64)> = tx.query_row("SELECT step_id,label,accepted_sequence FROM task_steps WHERE task_id=?1 ORDER BY accepted_sequence DESC LIMIT 1", [id], |r| Ok((r.get(0)?,r.get(1)?,r.get(2)?))).optional().map_err(storage)?;
        tx.commit().map_err(storage)?;
        let current_step = match step {
            Some((step_id, label, accepted)) => Some(StepDeclaration {
                step_id,
                label,
                accepted_sequence: u64::try_from(accepted)
                    .map_err(|_| Error::StorageUnavailable)?,
            }),
            None => None,
        };
        let observation = match (
            observation_step_id,
            observation_result_value,
            observation_summary,
        ) {
            (Some(step_id), Some(result), Some(summary)) => Some(TaskObservation {
                step_id,
                result: observation_result(&result)?,
                summary,
            }),
            (None, None, None) => None,
            _ => return Err(Error::StorageUnavailable),
        };
        Ok((
            Task {
                id: id.into(),
                owner_agent_id,
                name,
                source: task_source(&source)?,
                status: status(&state)?,
                sequence: u64::try_from(sequence).map_err(|_| Error::StorageUnavailable)?,
            },
            TaskPresentation {
                current_step,
                observation,
                next_intent,
            },
        ))
    }

    fn events_with_steps(
        &mut self,
        id: &str,
        after: u64,
        limit: usize,
    ) -> Result<Vec<TaskEventRecord>, Error> {
        if !(1..=100).contains(&limit) {
            return Err(Error::InvalidInput);
        }
        if after >= i64::MAX as u64 {
            return Ok(vec![]);
        }
        let mut stmt = self.0.prepare("SELECT e.previous,e.state,e.sequence,s.step_id,s.label,s.accepted_sequence,a.step_id,a.attempt_id,a.worker_instance_id,a.host_session_id,a.phase,a.action_succeeded,a.observe_valid,a.unknown_reason,e.wait_reason,m.version,m.item_count,c.confirmation_id,c.result_sequence,c.manifest_version,c.comment,c.confirmed_by,p.payload,h.attempt_id,h.control_id,h.kind,h.accepted_sequence,h.stopped_sequence,f.control_id,f.phase,f.failure,g.payload,t.owner_agent_id,ast.step_id,ast.attempt_id,ast.worker_instance_id,ast.host_session_id,v.verification_id,v.observation_sequence,v.outcome FROM events e JOIN tasks t ON t.id=e.task_id LEFT JOIN task_steps s ON s.task_id=e.task_id AND s.accepted_sequence=e.sequence LEFT JOIN task_attempts a ON a.task_id=e.task_id AND a.result_sequence=e.sequence LEFT JOIN task_attempts ast ON ast.task_id=e.task_id AND ast.accepted_sequence=e.sequence LEFT JOIN task_artifact_manifests m ON m.task_id=e.task_id AND m.created_sequence=e.sequence LEFT JOIN task_user_confirmations c ON c.task_id=e.task_id AND c.confirmation_sequence=e.sequence LEFT JOIN task_presentation_events p ON p.task_id=e.task_id AND p.sequence=e.sequence AND p.kind='observation' LEFT JOIN task_controls h ON h.task_id=e.task_id AND (h.accepted_sequence=e.sequence OR h.stopped_sequence=e.sequence) LEFT JOIN task_focus_events f ON f.task_id=e.task_id AND f.sequence=e.sequence LEFT JOIN task_presentation_events g ON g.task_id=e.task_id AND g.sequence=e.sequence AND g.kind='source' LEFT JOIN task_goal_verifications v ON v.task_id=e.task_id AND v.verified_sequence=e.sequence WHERE e.task_id=?1 AND e.sequence>?2 ORDER BY e.sequence LIMIT ?3").map_err(storage)?;
        let rows = stmt
            .query_map(params![id, after as i64, limit as i64], |r| {
                Ok((
                    r.get::<_, String>(0)?,
                    r.get::<_, String>(1)?,
                    r.get::<_, i64>(2)?,
                    r.get::<_, Option<String>>(3)?,
                    r.get::<_, Option<String>>(4)?,
                    r.get::<_, Option<i64>>(5)?,
                    r.get::<_, Option<String>>(6)?,
                    r.get::<_, Option<String>>(7)?,
                    r.get::<_, Option<String>>(8)?,
                    r.get::<_, Option<String>>(9)?,
                    r.get::<_, Option<String>>(10)?,
                    r.get::<_, Option<i64>>(11)?,
                    r.get::<_, Option<i64>>(12)?,
                    r.get::<_, Option<String>>(13)?,
                    r.get::<_, Option<String>>(14)?,
                    r.get::<_, Option<i64>>(15)?,
                    r.get::<_, Option<i64>>(16)?,
                    r.get::<_, Option<String>>(17)?,
                    r.get::<_, Option<i64>>(18)?,
                    r.get::<_, Option<i64>>(19)?,
                    r.get::<_, Option<String>>(20)?,
                    r.get::<_, Option<String>>(21)?,
                    r.get::<_, Option<String>>(22)?,
                    r.get::<_, Option<String>>(23)?,
                    r.get::<_, Option<String>>(24)?,
                    r.get::<_, Option<String>>(25)?,
                    r.get::<_, Option<i64>>(26)?,
                    r.get::<_, Option<i64>>(27)?,
                    r.get::<_, Option<String>>(28)?,
                    r.get::<_, Option<String>>(29)?,
                    r.get::<_, Option<String>>(30)?,
                    r.get::<_, Option<String>>(31)?,
                    r.get::<_, String>(32)?,
                    r.get::<_, Option<String>>(33)?,
                    r.get::<_, Option<String>>(34)?,
                    r.get::<_, Option<String>>(35)?,
                    r.get::<_, Option<String>>(36)?,
                    r.get::<_, Option<String>>(37)?,
                    r.get::<_, Option<i64>>(38)?,
                    r.get::<_, Option<String>>(39)?,
                ))
            })
            .map_err(storage)?;
        rows.map(|row| {
            let (
                previous,
                next,
                sequence,
                step_id,
                label,
                accepted,
                result_step,
                attempt_id,
                worker_id,
                host_id,
                phase,
                action_succeeded,
                observe_valid,
                unknown_reason,
                wait_reason,
                manifest_version,
                manifest_item_count,
                confirmation_id,
                result_sequence,
                confirmation_manifest_version,
                confirmation_comment,
                confirmed_by,
                observation_payload,
                control_attempt_id,
                control_id,
                control_kind,
                control_accepted,
                control_stopped,
                focus_control_id,
                focus_phase,
                focus_failure_value,
                source_payload,
                owner_agent_id,
                started_step_id,
                started_attempt_id,
                started_worker_id,
                started_host_id,
                verification_id,
                verification_observation_sequence,
                verification_outcome,
            ) = row.map_err(storage)?;
            let creation_event = source_payload
                .as_deref()
                .map(|payload| {
                    if sequence != 1 || !yonder_application::valid_id(&owner_agent_id) {
                        return Err(Error::StorageUnavailable);
                    }
                    Ok(yonder_application::CreationEventRecord {
                        owner_agent_id: owner_agent_id.clone(),
                        source: stored_source(payload)?,
                    })
                })
                .transpose()?;
            let step_declaration = match (step_id, label, accepted) {
                (Some(step_id), Some(label), Some(accepted)) => Some(StepDeclaration {
                    step_id,
                    label,
                    accepted_sequence: u64::try_from(accepted)
                        .map_err(|_| Error::StorageUnavailable)?,
                }),
                (None, None, None) => None,
                _ => return Err(Error::StorageUnavailable),
            };
            let attempt_started = match (
                started_step_id,
                started_attempt_id,
                started_worker_id,
                started_host_id,
            ) {
                (
                    Some(step_id),
                    Some(attempt_id),
                    Some(worker_instance_id),
                    Some(host_session_id),
                ) => {
                    if [
                        step_id.as_str(),
                        attempt_id.as_str(),
                        worker_instance_id.as_str(),
                        host_session_id.as_str(),
                    ]
                    .iter()
                    .any(|value| !yonder_application::valid_id(value))
                    {
                        return Err(Error::StorageUnavailable);
                    }
                    Some(yonder_application::AttemptStartedRecord {
                        step_id,
                        attempt_id,
                        worker_instance_id,
                        host_session_id,
                    })
                }
                (None, None, None, None) => None,
                _ => return Err(Error::StorageUnavailable),
            };
            let attempt_result = match (
                result_step,
                attempt_id,
                worker_id,
                host_id,
                phase,
                action_succeeded,
                observe_valid,
                unknown_reason,
            ) {
                (
                    Some(step_id),
                    Some(attempt_id),
                    Some(worker_instance_id),
                    Some(host_session_id),
                    Some(phase),
                    action,
                    observe,
                    reason_value,
                ) => {
                    let conclusion =
                        match (phase.as_str(), action, observe, reason_value.as_deref()) {
                            ("observed" | "stopped", Some(value), Some(1), None) => {
                                AttemptConclusion::Observed {
                                    action_succeeded: value != 0,
                                }
                            }
                            ("unknown" | "stopped", None, Some(0), Some(value)) => {
                                AttemptConclusion::Unknown {
                                    reason: parse_reason(value)?,
                                }
                            }
                            _ => return Err(Error::StorageUnavailable),
                        };
                    Some(AttemptResultRecord {
                        task_id: id.into(),
                        step_id,
                        attempt_id,
                        worker_instance_id,
                        host_session_id,
                        conclusion,
                        result_sequence: u64::try_from(sequence)
                            .map_err(|_| Error::StorageUnavailable)?,
                    })
                }
                (None, None, None, None, None, None, None, None) => None,
                _ => return Err(Error::StorageUnavailable),
            };
            let artifact_manifest = match (manifest_version, manifest_item_count) {
                (Some(version), Some(item_count)) => {
                    Some(yonder_application::TaskArtifactManifest {
                        task_id: id.to_owned(),
                        version: u64::try_from(version).map_err(|_| Error::StorageUnavailable)?,
                        item_count: u16::try_from(item_count)
                            .map_err(|_| Error::StorageUnavailable)?,
                    })
                }
                (None, None) => None,
                _ => return Err(Error::StorageUnavailable),
            };
            let user_confirmation = match (
                confirmation_id,
                result_sequence,
                confirmation_manifest_version,
                confirmation_comment,
                confirmed_by,
            ) {
                (
                    Some(confirmation_id),
                    Some(result_sequence),
                    Some(manifest_version),
                    comment,
                    Some(confirmed_by),
                ) => Some(yonder_application::TaskUserConfirmation {
                    task_id: id.to_owned(),
                    confirmation_id,
                    result_sequence: u64::try_from(result_sequence)
                        .map_err(|_| Error::StorageUnavailable)?,
                    manifest_version: u64::try_from(manifest_version)
                        .map_err(|_| Error::StorageUnavailable)?,
                    comment,
                    confirmed_by,
                }),
                (None, None, None, None, None) => None,
                _ => return Err(Error::StorageUnavailable),
            };
            let observation = observation_payload
                .as_deref()
                .map(stored_observation)
                .transpose()?;
            let control_event = match (
                control_attempt_id,
                control_id,
                control_kind,
                control_accepted,
                control_stopped,
            ) {
                (Some(attempt_id), Some(control_id), Some(kind), Some(accepted), stopped) => {
                    if accepted <= 0
                        || stopped.is_some_and(|value| value < accepted)
                        || !yonder_application::valid_id(&attempt_id)
                        || !yonder_application::valid_id(&control_id)
                    {
                        return Err(Error::StorageUnavailable);
                    }
                    let kind = match kind.as_str() {
                        "pause" => ControlKind::Pause,
                        "cancel" => ControlKind::Cancel,
                        "takeover" => ControlKind::Takeover,
                        _ => return Err(Error::StorageUnavailable),
                    };
                    let phase = if stopped == Some(sequence) {
                        ControlPhase::Stopped
                    } else if accepted == sequence {
                        ControlPhase::Pending
                    } else {
                        return Err(Error::StorageUnavailable);
                    };
                    Some(yonder_application::ControlEventRecord {
                        attempt_id,
                        control_id,
                        kind,
                        phase,
                    })
                }
                (None, None, None, None, None) => None,
                _ => return Err(Error::StorageUnavailable),
            };
            let focus_event = match (focus_control_id, focus_phase, focus_failure_value) {
                (Some(control_id), Some(phase), failure) => {
                    if !yonder_application::valid_id(&control_id) {
                        return Err(Error::StorageUnavailable);
                    }
                    let (phase, failure) = match (phase.as_str(), failure.as_deref()) {
                        ("locating", None) => (FocusPhase::Locating, None),
                        ("focused", None) => (FocusPhase::Focused, None),
                        ("failed", Some(value)) => {
                            (FocusPhase::Failed, Some(parse_focus_failure(value)?))
                        }
                        _ => return Err(Error::StorageUnavailable),
                    };
                    Some(yonder_application::FocusEventRecord {
                        control_id,
                        phase,
                        failure,
                    })
                }
                (None, None, None) => None,
                _ => return Err(Error::StorageUnavailable),
            };
            let goal_verification=match(verification_id,verification_observation_sequence,verification_outcome){
                (Some(verification_id),Some(observation_sequence),Some(outcome))=>Some(GoalVerificationRecord{task_id:id.into(),verification_id,observation_sequence:u64::try_from(observation_sequence).map_err(|_|Error::StorageUnavailable)?,verified_sequence:u64::try_from(sequence).map_err(|_|Error::StorageUnavailable)?,outcome:match outcome.as_str(){"achieved"=>GoalVerificationOutcome::Achieved,"not-achieved"=>GoalVerificationOutcome::NotAchieved,_=>return Err(Error::StorageUnavailable)}}),
                (None,None,None)=>None,
                _=>return Err(Error::StorageUnavailable),
            };
            Ok(TaskEventRecord {
                transition: Transition {
                    previous: status(&previous)?,
                    next: status(&next)?,
                    sequence: u64::try_from(sequence).map_err(|_| Error::StorageUnavailable)?,
                },
                creation_event,
                step_declaration,
                attempt_started,
                attempt_result,
                observation,
                control_event,
                focus_event,
                wait_reason,
                artifact_manifest,
                user_confirmation,
                goal_verification,
            })
        })
        .collect()
    }

    fn register(
        &mut self,
        owner: &str,
        key: &str,
        description: &str,
        task_name: Option<&str>,
        source: TaskSource,
    ) -> Result<Task, Error> {
        self.register_with_outcome(owner, key, description, task_name, source)
            .map(|outcome| outcome.task)
    }

    fn register_with_outcome(
        &mut self,
        owner: &str,
        key: &str,
        description: &str,
        task_name: Option<&str>,
        source: TaskSource,
    ) -> Result<yonder_application::RegistrationOutcome, Error> {
        if [owner, key].iter().any(|id| {
            id.is_empty()
                || id.len() > 128
                || !id
                    .bytes()
                    .all(|c| c.is_ascii_alphanumeric() || b"_-".contains(&c))
        }) || description.trim().is_empty()
            || description.len() > 4096
            || task_name.is_some_and(|name| !yonder_application::valid_task_name(name))
        {
            return Err(Error::InvalidInput);
        }
        let tx = self
            .0
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(storage)?;
        let existing: Option<(String,String,String,i64,Option<String>,Option<String>,String)> = tx.query_row(
            "SELECT c.task_id,c.description,t.state,t.sequence,c.name,t.name,t.source FROM task_creations c JOIN tasks t ON t.id=c.task_id WHERE c.owner_agent_id=?1 AND c.idempotency_key=?2",
            params![owner,key], |r| Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?,r.get(4)?,r.get(5)?,r.get(6)?)),
        ).optional().map_err(storage)?;
        let (task, created) = if let Some((
            id,
            original,
            state,
            sequence,
            original_name,
            current_name,
            current_source,
        )) = existing
        {
            if original != description
                || original_name.as_deref() != task_name
                || current_source != source_name(source)
            {
                return Err(Error::IdempotencyConflict);
            }
            (
                Task {
                    id,
                    owner_agent_id: owner.into(),
                    name: current_name,
                    source: task_source(&current_source)?,
                    status: status(&state)?,
                    sequence: u64::try_from(sequence).map_err(|_| Error::StorageUnavailable)?,
                },
                false,
            )
        } else {
            Self::ensure_audit_capacity(&tx)?;
            let id: String = tx
                .query_row("SELECT 'task_' || lower(hex(randomblob(16)))", [], |r| {
                    r.get(0)
                })
                .map_err(storage)?;
            let created_at = next_created_at(&tx)?;
            tx.execute("INSERT INTO tasks(id,owner_agent_id,state,sequence,name,source,created_at) VALUES (?1,?2,'created',1,?3,?4,?5)", params![id,owner,task_name,source_name(source),created_at]).map_err(storage)?;
            tx.execute("INSERT INTO events(task_id,sequence,previous,state) VALUES (?1,1,'created','created')", [&id]).map_err(storage)?;
            tx.execute("INSERT INTO outbox(task_id,sequence) VALUES (?1,1)", [&id])
                .map_err(storage)?;
            tx.execute("INSERT INTO task_presentation_events(task_id,sequence,kind,payload) VALUES (?1,1,'source',?2)", params![id,format!(r#"{{"source":"{}"}}"#, source_name(source))]).map_err(storage)?;
            tx.execute("INSERT INTO task_creations(owner_agent_id,idempotency_key,task_id,description,name) VALUES (?1,?2,?3,?4,?5)", params![owner,key,id,description,task_name]).map_err(storage)?;
            (
                Task {
                    id,
                    owner_agent_id: owner.into(),
                    name: task_name.map(str::to_owned),
                    source,
                    status: Status::Created,
                    sequence: 1,
                },
                true,
            )
        };
        tx.commit().map_err(storage)?;
        Ok(yonder_application::RegistrationOutcome { task, created })
    }

    fn list(
        &mut self,
        owner: Option<&str>,
        after: Option<&str>,
        include_finished: bool,
        limit: usize,
    ) -> Result<Vec<Task>, Error> {
        if !(1..=101).contains(&limit) {
            return Err(Error::InvalidInput);
        }
        let mut stmt = self.0.prepare("SELECT id,state,sequence,owner_agent_id,name,source FROM tasks WHERE id > ?1 COLLATE BINARY AND (?2 OR state NOT IN ('completed','failed','cancelled')) AND (?4 IS NULL OR owner_agent_id=?4) ORDER BY id COLLATE BINARY LIMIT ?3").map_err(storage)?;
        let rows = stmt
            .query_map(
                params![after.unwrap_or(""), include_finished, limit as i64, owner],
                |r| {
                    Ok((
                        r.get::<_, String>(0)?,
                        r.get::<_, String>(1)?,
                        r.get::<_, i64>(2)?,
                        r.get::<_, String>(3)?,
                        r.get::<_, Option<String>>(4)?,
                        r.get::<_, String>(5)?,
                    ))
                },
            )
            .map_err(storage)?;
        rows.map(|row| {
            let (id, state, sequence, owner_agent_id, name, source) = row.map_err(storage)?;
            Ok(Task {
                id,
                owner_agent_id,
                name,
                source: task_source(&source)?,
                status: status(&state)?,
                sequence: u64::try_from(sequence).map_err(|_| Error::StorageUnavailable)?,
            })
        })
        .collect()
    }
    fn list_running(
        &mut self,
        owner: Option<&str>,
        after: Option<&str>,
        limit: usize,
    ) -> Result<Vec<Task>, Error> {
        if !(1..=101).contains(&limit) {
            return Err(Error::InvalidInput);
        }
        let mut stmt = self.0.prepare("SELECT id,state,sequence,owner_agent_id,name,source FROM tasks WHERE id > ?1 COLLATE BINARY AND state='running' AND (?3 IS NULL OR owner_agent_id=?3) ORDER BY id COLLATE BINARY LIMIT ?2").map_err(storage)?;
        let rows = stmt
            .query_map(params![after.unwrap_or(""), limit as i64, owner], |r| {
                Ok((
                    r.get::<_, String>(0)?,
                    r.get::<_, String>(1)?,
                    r.get::<_, i64>(2)?,
                    r.get::<_, String>(3)?,
                    r.get::<_, Option<String>>(4)?,
                    r.get::<_, String>(5)?,
                ))
            })
            .map_err(storage)?;
        rows.map(|row| {
            let (id, state, sequence, owner_agent_id, name, source) = row.map_err(storage)?;
            Ok(Task {
                id,
                owner_agent_id,
                name,
                source: task_source(&source)?,
                status: status(&state)?,
                sequence: u64::try_from(sequence).map_err(|_| Error::StorageUnavailable)?,
            })
        })
        .collect()
    }
    fn list_newest(
        &mut self,
        owner: Option<&str>,
        after: Option<&str>,
        include_finished: bool,
        limit: usize,
    ) -> Result<Vec<Task>, Error> {
        if !(1..=101).contains(&limit) {
            return Err(Error::InvalidInput);
        }
        let mut stmt = self.0.prepare("SELECT id,state,sequence,owner_agent_id,name,source FROM tasks WHERE (?1 IS NULL OR created_at < (SELECT created_at FROM tasks WHERE id=?1) OR (created_at = (SELECT created_at FROM tasks WHERE id=?1) AND id < ?1 COLLATE BINARY)) AND (?2 OR state NOT IN ('completed','failed','cancelled')) AND (?4 IS NULL OR owner_agent_id=?4) ORDER BY created_at DESC, id COLLATE BINARY DESC LIMIT ?3").map_err(storage)?;
        let rows = stmt
            .query_map(params![after, include_finished, limit as i64, owner], |r| {
                Ok((
                    r.get::<_, String>(0)?,
                    r.get::<_, String>(1)?,
                    r.get::<_, i64>(2)?,
                    r.get::<_, String>(3)?,
                    r.get::<_, Option<String>>(4)?,
                    r.get::<_, String>(5)?,
                ))
            })
            .map_err(storage)?;
        rows.map(|row| {
            let (id, state, sequence, owner_agent_id, name, source) = row.map_err(storage)?;
            Ok(Task {
                id,
                owner_agent_id,
                name,
                source: task_source(&source)?,
                status: status(&state)?,
                sequence: u64::try_from(sequence).map_err(|_| Error::StorageUnavailable)?,
            })
        })
        .collect()
    }
    fn list_running_newest(
        &mut self,
        owner: Option<&str>,
        after: Option<&str>,
        limit: usize,
    ) -> Result<Vec<Task>, Error> {
        if !(1..=101).contains(&limit) {
            return Err(Error::InvalidInput);
        }
        let mut stmt = self.0.prepare("SELECT id,state,sequence,owner_agent_id,name,source FROM tasks WHERE (?1 IS NULL OR created_at < (SELECT created_at FROM tasks WHERE id=?1) OR (created_at = (SELECT created_at FROM tasks WHERE id=?1) AND id < ?1 COLLATE BINARY)) AND state='running' AND (?3 IS NULL OR owner_agent_id=?3) ORDER BY created_at DESC, id COLLATE BINARY DESC LIMIT ?2").map_err(storage)?;
        let rows = stmt
            .query_map(params![after, limit as i64, owner], |r| {
                Ok((
                    r.get::<_, String>(0)?,
                    r.get::<_, String>(1)?,
                    r.get::<_, i64>(2)?,
                    r.get::<_, String>(3)?,
                    r.get::<_, Option<String>>(4)?,
                    r.get::<_, String>(5)?,
                ))
            })
            .map_err(storage)?;
        rows.map(|row| {
            let (id, state, sequence, owner_agent_id, name, source) = row.map_err(storage)?;
            Ok(Task {
                id,
                owner_agent_id,
                name,
                source: task_source(&source)?,
                status: status(&state)?,
                sequence: u64::try_from(sequence).map_err(|_| Error::StorageUnavailable)?,
            })
        })
        .collect()
    }
    fn create(
        &mut self,
        id: &str,
        owner_agent_id: &str,
        source: TaskSource,
    ) -> Result<Task, Error> {
        if [id, owner_agent_id].iter().any(|id| {
            id.is_empty()
                || id.len() > 128
                || !id
                    .bytes()
                    .all(|c| c.is_ascii_alphanumeric() || b"_-".contains(&c))
        }) {
            return Err(Error::InvalidInput);
        }
        let tx = self
            .0
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(storage)?;
        let exists: bool = tx
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM tasks WHERE id=?1)",
                [id],
                |r| r.get(0),
            )
            .map_err(storage)?;
        if exists {
            return Err(Error::Conflict);
        }
        let created_at = next_created_at(&tx)?;
        tx.execute("INSERT INTO tasks(id,owner_agent_id,state,sequence,source,created_at) VALUES (?1,?2,'created',1,?3,?4)", params![id,owner_agent_id,source_name(source),created_at]).map_err(storage)?;
        tx.execute(
            "INSERT INTO events(task_id,sequence,previous,state) VALUES (?1,1,'created','created')",
            [id],
        )
        .map_err(storage)?;
        tx.execute("INSERT INTO outbox(task_id,sequence) VALUES (?1,1)", [id])
            .map_err(storage)?;
        tx.execute("INSERT INTO task_presentation_events(task_id,sequence,kind,payload) VALUES (?1,1,'source',?2)", params![id,format!(r#"{{"source":"{}"}}"#, source_name(source))]).map_err(storage)?;
        tx.commit().map_err(storage)?;
        Ok(Task {
            id: id.into(),
            owner_agent_id: owner_agent_id.into(),
            name: None,
            source,
            status: Status::Created,
            sequence: 1,
        })
    }

    fn get(&mut self, id: &str) -> Result<Task, Error> {
        let row: Option<(String, i64, String, Option<String>, String)> = self
            .0
            .query_row(
                "SELECT state,sequence,owner_agent_id,name,source FROM tasks WHERE id=?1",
                [id],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?)),
            )
            .optional()
            .map_err(storage)?;
        let (state, sequence, owner_agent_id, name, source) = row.ok_or(Error::NotFound)?;
        Ok(Task {
            id: id.into(),
            owner_agent_id,
            name,
            source: task_source(&source)?,
            status: status(&state)?,
            sequence: u64::try_from(sequence).map_err(|_| Error::StorageUnavailable)?,
        })
    }

    fn events(&mut self, id: &str, after: u64, limit: usize) -> Result<Vec<Transition>, Error> {
        if !(1..=100).contains(&limit) {
            return Err(Error::InvalidInput);
        }
        if after >= i64::MAX as u64 {
            return Ok(vec![]);
        }
        let mut stmt = self.0.prepare("SELECT previous,state,sequence FROM events WHERE task_id=?1 AND sequence>?2 ORDER BY sequence LIMIT ?3").map_err(storage)?;
        let rows = stmt
            .query_map(params![id, after as i64, limit as i64], |r| {
                Ok((
                    r.get::<_, String>(0)?,
                    r.get::<_, String>(1)?,
                    r.get::<_, i64>(2)?,
                ))
            })
            .map_err(storage)?;
        rows.map(|row| {
            let (previous, next, sequence) = row.map_err(storage)?;
            Ok(Transition {
                previous: status(&previous)?,
                next: status(&next)?,
                sequence: u64::try_from(sequence).map_err(|_| Error::StorageUnavailable)?,
            })
        })
        .collect()
    }

    fn running(&mut self, limit: usize) -> Result<Vec<Task>, Error> {
        if !(1..=100).contains(&limit) {
            return Err(Error::InvalidInput);
        }
        let mut stmt = self.0.prepare("SELECT id,sequence,owner_agent_id,name,source FROM tasks WHERE state='running' ORDER BY id LIMIT ?1").map_err(storage)?;
        let rows = stmt
            .query_map([limit as i64], |r| {
                Ok((
                    r.get::<_, String>(0)?,
                    r.get::<_, i64>(1)?,
                    r.get::<_, String>(2)?,
                    r.get::<_, Option<String>>(3)?,
                    r.get::<_, String>(4)?,
                ))
            })
            .map_err(storage)?;
        rows.map(|row| {
            let (id, sequence, owner_agent_id, name, source) = row.map_err(storage)?;
            Ok(Task {
                id,
                owner_agent_id,
                name,
                source: task_source(&source)?,
                status: Status::Running,
                sequence: u64::try_from(sequence).map_err(|_| Error::StorageUnavailable)?,
            })
        })
        .collect()
    }

    fn commit(&mut self, id: &str, expected: u64, change: Transition) -> Result<(), Error> {
        if expected == 0 || expected >= i64::MAX as u64 || change.sequence != expected + 1 {
            return Err(Error::InvalidInput);
        }
        let legal = [
            Action::Start,
            Action::Pause,
            Action::WaitForUser,
            Action::Resume,
            Action::Interrupt,
            Action::Complete,
            Action::Fail,
            Action::Cancel,
        ]
        .iter()
        .any(|action| change.previous.transition(expected, *action) == Ok(change));
        if !legal {
            return Err(Error::InvalidInput);
        }
        let tx = self
            .0
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(storage)?;
        let changed = tx
            .execute(
                "UPDATE tasks SET state=?1,sequence=?2 WHERE id=?3 AND sequence=?4 AND state=?5",
                params![
                    name(change.next),
                    change.sequence as i64,
                    id,
                    expected as i64,
                    name(change.previous)
                ],
            )
            .map_err(storage)?;
        if changed != 1 {
            return Err(Error::Conflict);
        }
        tx.execute(
            "INSERT INTO events(task_id,sequence,previous,state) VALUES (?1,?2,?3,?4)",
            params![
                id,
                change.sequence as i64,
                name(change.previous),
                name(change.next)
            ],
        )
        .map_err(storage)?;
        tx.execute(
            "INSERT INTO outbox(task_id,sequence) VALUES (?1,?2)",
            params![id, change.sequence as i64],
        )
        .map_err(storage)?;
        tx.commit().map_err(storage)
    }

    fn supports_running_filter(&self) -> bool {
        true
    }
    fn supports_newest_first(&self) -> bool {
        true
    }
    fn supports_registration(&self) -> bool {
        true
    }
    fn supports_step_declarations(&self) -> bool {
        true
    }
    fn supports_presentation(&self) -> bool {
        true
    }
}
