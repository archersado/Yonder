#[cfg(target_os = "macos")]
fn main() {
    use std::{path::Path, time::Duration};
    use yonder_adapters::{ego_lite::EgoLiteBridge, task_store::SqliteTaskStore};
    use yonder_application::admission::{Admission, Outcome, Resource, start_attempt};
    use yonder_application::browser_use::{
        BrowserAction, BrowserOutcome, BrowserUsePort, record_outcome,
    };
    use yonder_application::{
        AttemptPhase, AuthContext, ExecutionAttempt, TaskSource, TaskStore, advance_after_observe,
        prepare_next_attempt, register,
    };
    let cli = std::env::args().nth(1).expect("ego-browser绝对路径");
    let directory = std::env::temp_dir().join(format!("yonda-ego-persist-{}", std::process::id()));
    std::fs::create_dir(&directory).unwrap();
    let database = directory.join("tasks.db");
    let bridge = EgoLiteBridge::new(Path::new(&cli), Duration::from_secs(30)).unwrap();
    let mut store = SqliteTaskStore::open_unencrypted(&database).unwrap();
    let task = register(
        &mut store,
        AuthContext::Agent("verify"),
        "ego-persist",
        "verify",
        Some("BUA引用验证"),
        TaskSource::LocalAgent,
    )
    .unwrap();
    let (task, _) = store
        .declare_step("verify", &task.id, task.sequence, "create", "创建空间")
        .unwrap();
    let make_attempt = |step: &str, id: &str| ExecutionAttempt {
        task_id: task.id.clone(),
        step_id: step.into(),
        attempt_id: id.into(),
        worker_instance_id: "worker".into(),
        host_session_id: "host".into(),
        phase: AttemptPhase::Prepared,
        accepted_sequence: 0,
    };
    let gate = Admission::new(1).unwrap();
    let (_, first, permit) = start_attempt(
        &mut store,
        &gate,
        &make_attempt("create", "attempt-1"),
        task.sequence,
        &[Resource::Browser],
    )
    .unwrap();
    let created = bridge.dispatch(
        &first,
        &BrowserAction::Create {
            name: format!("Yonder 持久化验证 {}", std::process::id()),
        },
    );
    let external = match &created {
        BrowserOutcome::Observed(value) => value.external_task_ref.clone(),
        other => panic!("{other:?}"),
    };
    let (observed, _) = record_outcome(&mut store, &task.id, &first.attempt_id, &created).unwrap();
    assert_eq!(
        store
            .get_browser_reference(&task.id)
            .unwrap()
            .unwrap()
            .external_task_ref,
        external
    );
    drop(store);
    let mut store = SqliteTaskStore::open_unencrypted(&database).unwrap();
    assert_eq!(
        store
            .get_browser_reference(&task.id)
            .unwrap()
            .unwrap()
            .external_task_ref,
        external
    );
    let (advanced, _) = advance_after_observe(&mut store, &task.id, &first.attempt_id).unwrap();
    let (declared, _) = store
        .declare_step("verify", &task.id, advanced.sequence, "finish", "完成空间")
        .unwrap();
    let (_, second) = prepare_next_attempt(
        &mut store,
        &make_attempt("finish", "attempt-2"),
        declared.sequence,
    )
    .unwrap();
    let finished = bridge.dispatch(
        &second,
        &BrowserAction::Finish {
            external_task_ref: external.clone(),
        },
    );
    let (finished_task, _) =
        record_outcome(&mut store, &task.id, &second.attempt_id, &finished).unwrap();
    assert!(
        store
            .get_browser_reference(&task.id)
            .unwrap()
            .unwrap()
            .finished
    );
    let (advanced, _) = advance_after_observe(&mut store, &task.id, &second.attempt_id).unwrap();
    assert_eq!(
        permit
            .finish_after_stop(&mut store, advanced.sequence, Outcome::Completed)
            .ok()
            .unwrap()
            .status,
        yonder_application::Status::Completed
    );
    assert!(finished_task.sequence < advanced.sequence);
    drop(store);
    std::fs::remove_file(database).unwrap();
    std::fs::remove_file(directory.join("host.lock")).ok();
    std::fs::remove_dir_all(directory).unwrap();
    println!(
        r#"{{"external_task_ref":"{external}","persisted_after_reopen":true,"finished_persisted":true,"space_cleaned":true,"observed_sequence":{}}}"#,
        observed.sequence
    );
}

#[cfg(not(target_os = "macos"))]
fn main() {
    eprintln!("capability_unavailable");
    std::process::exit(2)
}
