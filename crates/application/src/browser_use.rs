use crate::{
    AttemptResultRecord, Error, ExecutionAttempt, Task, TaskStore,
    computer_use::{DispatchOutcome, UnknownReason, record_dispatch_outcome},
};
use crate::{
    AuthContext,
    admission::{Admission, Resource, start_execution},
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BrowserTaskRef {
    pub external_task_ref: String,
    pub ownership: String,
    pub managed_pages: usize,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BrowserReferenceRecord {
    pub task_id: String,
    pub external_task_ref: String,
    pub ownership: String,
    pub managed_pages: usize,
    pub finished: bool,
    pub updated_sequence: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum BrowserAction {
    Create { name: String },
    Observe { external_task_ref: String },
    HandOff { external_task_ref: String },
    TakeOver { external_task_ref: String },
    Finish { external_task_ref: String },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum BrowserOutcome {
    Observed(BrowserTaskRef),
    Finished { external_task_ref: String },
    Unknown(UnknownReason),
}

pub trait BrowserUsePort {
    fn dispatch(&self, attempt: &ExecutionAttempt, action: &BrowserAction) -> BrowserOutcome;
}

/// 只供可信 Application 编排；空间引用来自上一次 Bridge 结果，不能由 UI 伪造。
pub fn dispatch_prepared(
    store: &mut impl TaskStore,
    port: &(impl BrowserUsePort + ?Sized),
    task_id: &str,
    attempt_id: &str,
    action: &BrowserAction,
) -> Result<BrowserOutcome, Error> {
    if !crate::valid_id(task_id) || !crate::valid_id(attempt_id) {
        return Err(Error::InvalidInput);
    }
    if store
        .get_control(task_id)?
        .is_some_and(|control| control.phase == crate::ControlPhase::Pending)
    {
        return Err(Error::StopRequired);
    }
    let attempt = store
        .get_attempt(task_id)?
        .filter(|attempt| {
            attempt.attempt_id == attempt_id && attempt.phase == crate::AttemptPhase::Prepared
        })
        .ok_or(Error::Conflict)?;
    Ok(port.dispatch(&attempt, action))
}

/// 成功必须包含 Bridge 的后置观察；unknown 沿用既有不可重试结果语义。
pub fn record_outcome(
    store: &mut impl TaskStore,
    task_id: &str,
    attempt_id: &str,
    outcome: &BrowserOutcome,
) -> Result<(Task, AttemptResultRecord), Error> {
    if !crate::valid_id(task_id) || !crate::valid_id(attempt_id) {
        return Err(Error::InvalidInput);
    }
    let task = store.get(task_id)?;
    let attempt = store
        .get_attempt(task_id)?
        .filter(|attempt| attempt.attempt_id == attempt_id)
        .ok_or(Error::Conflict)?;
    match outcome {
        BrowserOutcome::Observed(reference)
            if valid_ref(&reference.external_task_ref)
                && valid_ownership(&reference.ownership)
                && reference.managed_pages <= 100 =>
        {
            store.record_browser_result(
                &attempt,
                task.sequence,
                &BrowserReferenceRecord {
                    task_id: task_id.into(),
                    external_task_ref: reference.external_task_ref.clone(),
                    ownership: reference.ownership.clone(),
                    managed_pages: reference.managed_pages,
                    finished: false,
                    updated_sequence: 0,
                },
            )
        }
        BrowserOutcome::Finished { external_task_ref } if valid_ref(external_task_ref) => {
            let previous = store
                .get_browser_reference(task_id)?
                .filter(|value| value.external_task_ref == *external_task_ref && !value.finished)
                .ok_or(Error::Conflict)?;
            store.record_browser_result(
                &attempt,
                task.sequence,
                &BrowserReferenceRecord {
                    finished: true,
                    updated_sequence: 0,
                    ..previous
                },
            )
        }
        BrowserOutcome::Unknown(reason) => record_dispatch_outcome(
            store,
            task_id,
            attempt_id,
            DispatchOutcome::Unknown(*reason),
        ),
        _ => record_dispatch_outcome(
            store,
            task_id,
            attempt_id,
            DispatchOutcome::Unknown(UnknownReason::InvalidResponse),
        ),
    }
}

pub fn valid_ref(value: &str) -> bool {
    value
        .strip_prefix("ego:")
        .is_some_and(|id| id.parse::<u64>().is_ok_and(|value| value > 0))
}

fn valid_ownership(value: &str) -> bool {
    matches!(value, "agent" | "agentDelegatedToUser" | "user")
}

pub fn execute_agent_action(
    store: &mut impl TaskStore,
    admission: &Admission,
    port: &(impl BrowserUsePort + ?Sized),
    auth: AuthContext<'_>,
    task_id: &str,
    expected: u64,
    operation: &str,
    host_session_id: &str,
) -> Result<(Task, BrowserReferenceRecord), Error> {
    let (task, step) = crate::get_with_step(store, auth, task_id)?;
    let step = step.ok_or(Error::StopRequired)?;
    if task.sequence != expected || !crate::valid_id(host_session_id) {
        return Err(Error::Conflict);
    }
    let previous = store.get_browser_reference(task_id)?;
    let action = match operation {
        "create" if previous.is_none() => BrowserAction::Create {
            name: task.name.clone().unwrap_or_else(|| task.id.clone()),
        },
        "observe" => BrowserAction::Observe {
            external_task_ref: previous
                .as_ref()
                .filter(|v| !v.finished)
                .ok_or(Error::StopRequired)?
                .external_task_ref
                .clone(),
        },
        "hand-off" => BrowserAction::HandOff {
            external_task_ref: previous
                .as_ref()
                .filter(|v| !v.finished)
                .ok_or(Error::StopRequired)?
                .external_task_ref
                .clone(),
        },
        "take-over" => BrowserAction::TakeOver {
            external_task_ref: previous
                .as_ref()
                .filter(|v| !v.finished)
                .ok_or(Error::StopRequired)?
                .external_task_ref
                .clone(),
        },
        "finish" => BrowserAction::Finish {
            external_task_ref: previous
                .as_ref()
                .filter(|v| !v.finished)
                .ok_or(Error::StopRequired)?
                .external_task_ref
                .clone(),
        },
        _ => return Err(Error::InvalidInput),
    };
    let attempt = ExecutionAttempt {
        task_id: task_id.into(),
        step_id: step.step_id,
        attempt_id: format!("attempt_{}", expected + 1),
        worker_instance_id: "ego_bridge".into(),
        host_session_id: host_session_id.into(),
        phase: crate::AttemptPhase::Prepared,
        accepted_sequence: 0,
    };
    let accepted = start_execution(
        store,
        admission,
        &task,
        &attempt,
        expected,
        &[Resource::Browser],
    )
    .map_err(|_| Error::StopRequired)?
    .1;
    let outcome = dispatch_prepared(store, port, task_id, &accepted.attempt_id, &action)?;
    let (result_task, _) = record_outcome(store, task_id, &accepted.attempt_id, &outcome)?;
    let reference = store
        .get_browser_reference(task_id)?
        .ok_or(Error::StopRequired)?;
    if operation == "finish" && reference.finished {
        let (advanced, _) = crate::advance_after_observe(store, task_id, &accepted.attempt_id)?;
        let completed =
            crate::transition(store, task_id, advanced.sequence, crate::Action::Complete)?;
        admission
            .release_task_after_stop(task_id)
            .map_err(|_| Error::StorageUnavailable)?;
        return Ok((completed, reference));
    }
    Ok((result_task, reference))
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Store {
        attempt: ExecutionAttempt,
        pending: bool,
    }
    impl TaskStore for Store {
        fn get_attempt(&mut self, _: &str) -> Result<Option<ExecutionAttempt>, Error> {
            Ok(Some(self.attempt.clone()))
        }
        fn get_control(&mut self, _: &str) -> Result<Option<crate::ControlRequestRecord>, Error> {
            Ok(self.pending.then(|| crate::ControlRequestRecord {
                task_id: "task".into(),
                attempt_id: "attempt".into(),
                control_id: "control".into(),
                kind: crate::ControlKind::Takeover,
                phase: crate::ControlPhase::Pending,
                accepted_sequence: 2,
                stopped_sequence: None,
                focus_phase: None,
                focus_failure: None,
            }))
        }
        fn create(&mut self, _: &str, _: &str, _: crate::TaskSource) -> Result<Task, Error> {
            Err(Error::StorageUnavailable)
        }
        fn get(&mut self, _: &str) -> Result<Task, Error> {
            Err(Error::StorageUnavailable)
        }
        fn list(
            &mut self,
            _: Option<&str>,
            _: Option<&str>,
            _: bool,
            _: usize,
        ) -> Result<Vec<Task>, Error> {
            Err(Error::StorageUnavailable)
        }
        fn running(&mut self, _: usize) -> Result<Vec<Task>, Error> {
            Err(Error::StorageUnavailable)
        }
        fn events(
            &mut self,
            _: &str,
            _: u64,
            _: usize,
        ) -> Result<Vec<yonder_domain::Transition>, Error> {
            Err(Error::StorageUnavailable)
        }
        fn commit(&mut self, _: &str, _: u64, _: yonder_domain::Transition) -> Result<(), Error> {
            Err(Error::StorageUnavailable)
        }
    }
    struct Port;
    impl BrowserUsePort for Port {
        fn dispatch(&self, _: &ExecutionAttempt, _: &BrowserAction) -> BrowserOutcome {
            BrowserOutcome::Observed(BrowserTaskRef {
                external_task_ref: "ego:1".into(),
                ownership: "agent".into(),
                managed_pages: 1,
            })
        }
    }

    #[test]
    fn only_numeric_ego_references_are_accepted() {
        assert!(valid_ref("ego:42"));
        for value in ["42", "ego:", "ego:-1", "ego:1/x"] {
            assert!(!valid_ref(value));
        }
    }

    #[test]
    fn prepared_attempt_dispatches_but_pending_control_stops_it() {
        let attempt = ExecutionAttempt {
            task_id: "task".into(),
            step_id: "step".into(),
            attempt_id: "attempt".into(),
            worker_instance_id: "worker".into(),
            host_session_id: "host".into(),
            phase: crate::AttemptPhase::Prepared,
            accepted_sequence: 1,
        };
        let action = BrowserAction::Observe {
            external_task_ref: "ego:1".into(),
        };
        let mut store = Store {
            attempt,
            pending: false,
        };
        assert!(matches!(
            dispatch_prepared(&mut store, &Port, "task", "attempt", &action),
            Ok(BrowserOutcome::Observed(_))
        ));
        store.pending = true;
        assert_eq!(
            dispatch_prepared(&mut store, &Port, "task", "attempt", &action),
            Err(Error::StopRequired)
        );
    }
}
