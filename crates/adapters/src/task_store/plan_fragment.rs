use super::*;

impl yonder_application::TaskPlanFragmentStore for SqliteTaskStore {
    fn supports_plan_fragments(&self) -> bool {
        true
    }
    fn submit_plan_fragment(
        &mut self,
        owner: &str,
        fragment: &PlanFragment,
    ) -> Result<Task, Error> {
        yonder_application::plan_fragment::validate(fragment)?;
        if !yonder_application::valid_id(owner) {
            return Err(Error::InvalidInput);
        }
        let json = serde_json::to_string(fragment).map_err(|_| Error::StorageUnavailable)?;
        let tx = self
            .0
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(storage)?;
        let row: Option<(String, i64, String, Option<String>, String)> = tx
            .query_row(
                "SELECT state,sequence,owner_agent_id,name,source FROM tasks WHERE id=?1",
                [&fragment.task_id],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?)),
            )
            .optional()
            .map_err(storage)?;
        let (state, sequence, actual_owner, name, source) = row.ok_or(Error::NotFound)?;
        if actual_owner != owner {
            return Err(Error::NotFound);
        }
        let task = Task {
            id: fragment.task_id.clone(),
            owner_agent_id: actual_owner.clone(),
            name,
            source: task_source(&source)?,
            status: status(&state)?,
            sequence: u64::try_from(sequence).map_err(|_| Error::StorageUnavailable)?,
        };
        let existing: Option<String> = tx.query_row(
            "SELECT fragment_json FROM task_plan_fragments WHERE task_id=?1 AND plan_id=?2 AND plan_version=?3",
            params![fragment.task_id,fragment.plan_id,fragment.plan_version as i64], |r|r.get(0)
        ).optional().map_err(storage)?;
        if let Some(existing) = existing {
            if existing == json {
                tx.commit().map_err(storage)?;
                return Ok(task);
            }
            return Err(Error::Conflict);
        }
        if !matches!(task.status, Status::Created | Status::Running)
            || task.sequence != fragment.expected_sequence
        {
            return Err(Error::Conflict);
        }
        let accepted = task
            .sequence
            .checked_add(1)
            .ok_or(Error::StorageUnavailable)?;
        if tx
            .execute(
                "UPDATE tasks SET sequence=?1 WHERE id=?2 AND owner_agent_id=?3 AND sequence=?4",
                params![
                    accepted as i64,
                    fragment.task_id,
                    owner,
                    task.sequence as i64
                ],
            )
            .map_err(storage)?
            != 1
        {
            return Err(Error::Conflict);
        }
        tx.execute(
            "INSERT INTO events(task_id,sequence,previous,state) VALUES(?1,?2,?3,?3)",
            params![fragment.task_id, accepted as i64, state],
        )
        .map_err(storage)?;
        tx.execute(
            "INSERT INTO outbox(task_id,sequence) VALUES(?1,?2)",
            params![fragment.task_id, accepted as i64],
        )
        .map_err(storage)?;
        tx.execute("INSERT INTO task_plan_fragments(task_id,plan_id,plan_version,owner_agent_id,accepted_sequence,fragment_json) VALUES(?1,?2,?3,?4,?5,?6)",params![fragment.task_id,fragment.plan_id,fragment.plan_version as i64,owner,accepted as i64,json]).map_err(storage)?;
        tx.commit().map_err(storage)?;
        Ok(Task {
            sequence: accepted,
            ..task
        })
    }
    fn get_plan_fragment(
        &mut self,
        task_id: &str,
        plan_id: &str,
        plan_version: u64,
    ) -> Result<Option<StoredPlanFragment>, Error> {
        let row: Option<(String,i64,i64,String)> = self.0.query_row(
            "SELECT owner_agent_id,accepted_sequence,current_slot,fragment_json FROM task_plan_fragments WHERE task_id=?1 AND plan_id=?2 AND plan_version=?3",
            params![task_id,plan_id,plan_version as i64], |r|Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?))
        ).optional().map_err(storage)?;
        row.map(|(owner, accepted, current, json)| {
            Ok(StoredPlanFragment {
                owner_agent_id: owner,
                accepted_sequence: u64::try_from(accepted)
                    .map_err(|_| Error::StorageUnavailable)?,
                current_slot: u16::try_from(current).map_err(|_| Error::StorageUnavailable)?,
                fragment: serde_json::from_str(&json).map_err(|_| Error::StorageUnavailable)?,
            })
        })
        .transpose()
    }
    fn advance_plan_fragment(
        &mut self,
        task_id: &str,
        plan_id: &str,
        plan_version: u64,
        slot: u16,
        expected: u64,
    ) -> Result<Task, Error> {
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
        let (state, sequence, owner, name, source) = row.ok_or(Error::NotFound)?;
        let current = u64::try_from(sequence).map_err(|_| Error::StorageUnavailable)?;
        if current != expected || state != "running" {
            return Err(Error::Conflict);
        }
        let changed=tx.execute("UPDATE task_plan_fragments SET current_slot=current_slot+1 WHERE task_id=?1 AND plan_id=?2 AND plan_version=?3 AND current_slot=?4",params![task_id,plan_id,plan_version as i64,slot as i64]).map_err(storage)?;
        if changed != 1 {
            return Err(Error::Conflict);
        }
        let next = expected.checked_add(1).ok_or(Error::StorageUnavailable)?;
        if tx
            .execute(
                "UPDATE tasks SET sequence=?1 WHERE id=?2 AND sequence=?3",
                params![next as i64, task_id, expected as i64],
            )
            .map_err(storage)?
            != 1
        {
            return Err(Error::Conflict);
        }
        tx.execute(
            "INSERT INTO events(task_id,sequence,previous,state) VALUES(?1,?2,?3,?3)",
            params![task_id, next as i64, state],
        )
        .map_err(storage)?;
        tx.execute(
            "INSERT INTO outbox(task_id,sequence) VALUES(?1,?2)",
            params![task_id, next as i64],
        )
        .map_err(storage)?;
        tx.commit().map_err(storage)?;
        Ok(Task {
            id: task_id.into(),
            owner_agent_id: owner,
            name,
            source: task_source(&source)?,
            status: Status::Running,
            sequence: next,
        })
    }
    fn hand_back_plan_fragment(
        &mut self,
        task_id: &str,
        plan_id: &str,
        plan_version: u64,
        expected: u64,
        reason: &str,
    ) -> Result<Task, Error> {
        if reason.is_empty()
            || reason.as_bytes().len() > 1024
            || reason
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
        let (state, sequence, owner, name, source) = row.ok_or(Error::NotFound)?;
        // AD-TM-21：Jev 可在第一个 CUA 动作启动前交回，但 plan.execute 已经
        // 启动产品执行闭环；交回必须让 created 原子进入 running，避免控制条
        // 与 Task Space 状态分裂。
        if !matches!(state.as_str(), "created" | "running")
            || u64::try_from(sequence).map_err(|_| Error::StorageUnavailable)? != expected
        {
            return Err(Error::Conflict);
        }
        let exists:i64=tx.query_row("SELECT count(*) FROM task_plan_fragments WHERE task_id=?1 AND plan_id=?2 AND plan_version=?3",params![task_id,plan_id,plan_version as i64],|r|r.get(0)).map_err(storage)?;
        if exists != 1 {
            return Err(Error::NotFound);
        }
        let next = expected.checked_add(1).ok_or(Error::StorageUnavailable)?;
        if tx.execute("UPDATE tasks SET state='running',sequence=?1,next_intent=?2 WHERE id=?3 AND sequence=?4",params![next as i64,reason,task_id,expected as i64]).map_err(storage)?!=1{return Err(Error::Conflict);}
        tx.execute(
            "INSERT INTO events(task_id,sequence,previous,state) VALUES(?1,?2,?3,'running')",
            params![task_id, next as i64, state],
        )
        .map_err(storage)?;
        tx.execute(
            "INSERT INTO outbox(task_id,sequence) VALUES(?1,?2)",
            params![task_id, next as i64],
        )
        .map_err(storage)?;
        // unknown 结论永久保留，但已经交回慢脑的 Driver attempt 必须释放步骤
        // 执行槽。以同一个 handback 序号关闭 attempt，归属 Agent 才能读取
        // unknown/Observe 后提交新的受限片段；这不会推进旧片段或重放副作用。
        tx.execute(
            "UPDATE task_attempts SET handback_sequence=?1 WHERE task_id=?2 AND phase='unknown' AND result_sequence IS NOT NULL AND handback_sequence IS NULL",
            params![next as i64, task_id],
        )
        .map_err(storage)?;
        tx.execute("INSERT INTO task_presentation_events(task_id,sequence,kind,payload) VALUES(?1,?2,'next-intent',?3)",params![task_id,next as i64,format!(r#"{{"next_intent":{}}}"#, serde_json::to_string(reason).map_err(|_| Error::StorageUnavailable)?) ]).map_err(storage)?;
        tx.commit().map_err(storage)?;
        Ok(Task {
            id: task_id.into(),
            owner_agent_id: owner,
            name,
            source: task_source(&source)?,
            status: Status::Running,
            sequence: next,
        })
    }
    fn supports_goal_verification(&self) -> bool { true }
    fn record_goal_verification(
        &mut self,
        task_id: &str,
        expected: u64,
        observation_sequence: u64,
        verification_id: &str,
        outcome: GoalVerificationOutcome,
    ) -> Result<(Task, GoalVerificationRecord), Error> {
        if !yonder_application::valid_id(task_id)
            || !yonder_application::valid_id(verification_id)
            || expected == 0
            || observation_sequence == 0
            || observation_sequence > expected
        { return Err(Error::InvalidInput); }
        let tx=self.0.transaction_with_behavior(TransactionBehavior::Immediate).map_err(storage)?;
        let row:Option<(String,i64,String,Option<String>,String)>=tx.query_row(
            "SELECT state,sequence,owner_agent_id,name,source FROM tasks WHERE id=?1",[task_id],
            |r|Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?,r.get(4)?))
        ).optional().map_err(storage)?;
        let (state,sequence,owner,name,source)=row.ok_or(Error::NotFound)?;
        if state!="running" || u64::try_from(sequence).map_err(|_|Error::StorageUnavailable)?!=expected{return Err(Error::Conflict);}
        let latest:Option<(i64,Option<i64>,Option<i64>)>=tx.query_row(
            "SELECT result_sequence,action_succeeded,observe_valid FROM task_attempts WHERE task_id=?1 AND result_sequence IS NOT NULL ORDER BY result_sequence DESC LIMIT 1",
            [task_id],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?))
        ).optional().map_err(storage)?;
        if !latest.is_some_and(|(result,action,observe)| result==observation_sequence as i64&&action==Some(1)&&observe==Some(1)){return Err(Error::StopRequired);}
        let next=expected.checked_add(1).ok_or(Error::StorageUnavailable)?;
        let outcome_name=match outcome{GoalVerificationOutcome::Achieved=>"achieved",GoalVerificationOutcome::NotAchieved=>"not-achieved"};
        if tx.execute("UPDATE tasks SET sequence=?1,next_intent=?2 WHERE id=?3 AND sequence=?4",params![next as i64,if outcome==GoalVerificationOutcome::NotAchieved{Some("目标未完成，已交回慢脑")}else{None::<&str>},task_id,expected as i64]).map_err(storage)?!=1{return Err(Error::Conflict);}
        tx.execute("INSERT INTO events(task_id,sequence,previous,state) VALUES(?1,?2,'running','running')",params![task_id,next as i64]).map_err(storage)?;
        tx.execute("INSERT INTO outbox(task_id,sequence) VALUES(?1,?2)",params![task_id,next as i64]).map_err(storage)?;
        tx.execute("INSERT INTO task_goal_verifications(task_id,verification_id,observation_sequence,verified_sequence,outcome) VALUES(?1,?2,?3,?4,?5)",params![task_id,verification_id,observation_sequence as i64,next as i64,outcome_name]).map_err(|error|if matches!(error,rusqlite::Error::SqliteFailure(_, _)){Error::Conflict}else{storage(error)})?;
        tx.commit().map_err(storage)?;
        let record=GoalVerificationRecord{task_id:task_id.into(),verification_id:verification_id.into(),observation_sequence,verified_sequence:next,outcome};
        Ok((Task{id:task_id.into(),owner_agent_id:owner,name,source:task_source(&source)?,status:Status::Running,sequence:next},record))
    }
    fn get_goal_verification(&mut self, task_id:&str)->Result<Option<GoalVerificationRecord>,Error>{
        let row:Option<(String,i64,i64,String)>=self.0.query_row(
            "SELECT verification_id,observation_sequence,verified_sequence,outcome FROM task_goal_verifications WHERE task_id=?1 ORDER BY verified_sequence DESC LIMIT 1",
            [task_id],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?))
        ).optional().map_err(storage)?;
        row.map(|(verification_id,observation,verified,outcome)|Ok(GoalVerificationRecord{
            task_id:task_id.into(),verification_id,observation_sequence:u64::try_from(observation).map_err(|_|Error::StorageUnavailable)?,verified_sequence:u64::try_from(verified).map_err(|_|Error::StorageUnavailable)?,outcome:match outcome.as_str(){"achieved"=>GoalVerificationOutcome::Achieved,"not-achieved"=>GoalVerificationOutcome::NotAchieved,_=>return Err(Error::StorageUnavailable)}
        })).transpose()
    }
}
