use std::{path::Path, time::{Duration, SystemTime, UNIX_EPOCH}};
use yonder_adapters::{cua::CuaWorker, task_store::SqliteTaskStore};
use yonder_application::{AttemptConclusion, AttemptPhase, AuthContext, ExecutionAttempt, admission::{Admission, Resource, start_attempt}, computer_use::{ComputerAction, WorkTarget, dispatch_prepared, record_dispatch_outcome}};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    assert_eq!(args.len(), 6, "node worker sdk pid window_id");
    let evidence=std::env::temp_dir().join(format!("yonda-cua-check-{}",std::process::id()));
    std::fs::create_dir_all(&evidence).unwrap();
    let worker = CuaWorker::new(Path::new(&args[1]), Path::new(&args[2]), Path::new(&args[3]), &evidence, Duration::from_secs(30)).unwrap();
    let database = std::env::temp_dir().join(format!("yonda-cu-check-{}-{}.db", std::process::id(), SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos()));
    let mut store = SqliteTaskStore::open_unencrypted(&database).unwrap();
    let task = yonder_application::create(&mut store, "cu_native_fixture", AuthContext::Agent("verification_agent")).unwrap();
    let (task, _) = yonder_application::declare_step(&mut store, AuthContext::Agent("verification_agent"), &task.id, task.sequence, "type_once", "隔离输入").unwrap();
    let attempt = ExecutionAttempt { task_id: task.id.clone(), step_id: "type_once".into(), attempt_id: "attempt_once".into(), worker_instance_id: "worker_once".into(), host_session_id: "host_fixture".into(), phase: AttemptPhase::Prepared, accepted_sequence: 0 };
    let admission = Admission::new(1).unwrap();
    let (_, accepted, permit) = start_attempt(&mut store, &admission, &attempt, task.sequence, &[Resource::Desktop]).unwrap();
    let outcome = dispatch_prepared(&mut store, &worker, &accepted.task_id, &accepted.attempt_id, &WorkTarget { pid: args[4].parse().unwrap(), window_id: args[5].parse().unwrap() }, &ComputerAction { tool_name:"type_text".into(), arguments_json:r#"{"text":"YONDER_SDK_INPUT_A","delivery_mode":"background"}"#.into() }).unwrap();
    let (_, result) = record_dispatch_outcome(&mut store, &accepted.task_id, &accepted.attempt_id, outcome).unwrap();
    let known_success = matches!(result.conclusion, AttemptConclusion::Observed { action_succeeded:true });
    assert!(admission.has_occupancy().unwrap());
    drop(permit);
    drop(store);
    let _ = std::fs::remove_file(database);
    println!("{{\"known_success\":{known_success},\"result_persisted\":true,\"occupancy_retained\":true}}");
    if !known_success { std::process::exit(1); }
}
