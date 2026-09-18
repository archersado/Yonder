#[cfg(target_os = "macos")]
fn main() {
    use std::{path::Path, time::Duration};
    use yonder_adapters::ego_lite::EgoLiteBridge;
    use yonder_application::{AttemptPhase, ExecutionAttempt, browser_use::{BrowserAction, BrowserOutcome, BrowserUsePort}};
    let cli = std::env::args().nth(1).expect("ego-browser绝对路径");
    let bridge = EgoLiteBridge::new(Path::new(&cli), Duration::from_secs(30)).unwrap();
    let attempt = ExecutionAttempt { task_id:"task".into(),step_id:"step".into(),attempt_id:"attempt".into(),worker_instance_id:"worker".into(),host_session_id:"host".into(),phase:AttemptPhase::Prepared,accepted_sequence:1 };
    let created = bridge.dispatch(&attempt,&BrowserAction::Create { name:format!("Yonder Bridge 验证 {}",std::process::id()) });
    let reference = match created { BrowserOutcome::Observed(value) => value.external_task_ref, other => panic!("创建失败: {other:?}") };
    for action in [BrowserAction::Observe { external_task_ref:reference.clone() }, BrowserAction::HandOff { external_task_ref:reference.clone() }, BrowserAction::TakeOver { external_task_ref:reference.clone() }] {
        let outcome = bridge.dispatch(&attempt,&action);
        assert!(matches!(outcome,BrowserOutcome::Observed(_)),"{action:?}: {outcome:?}");
    }
    assert_eq!(bridge.dispatch(&attempt,&BrowserAction::Finish { external_task_ref:reference.clone() }),BrowserOutcome::Finished { external_task_ref:reference.clone() });
    println!(r#"{{"external_task_ref":"{reference}","create":true,"observe":true,"hand_off":true,"take_over":true,"finish":true}}"#);
}

#[cfg(not(target_os = "macos"))]
fn main() { eprintln!("capability_unavailable"); std::process::exit(2); }
