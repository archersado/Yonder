#[cfg(target_os = "macos")]
fn main() {
    use std::{
        path::Path,
        time::{Duration, Instant},
    };
    use yonder_adapters::jev::MacosJevPort;
    use yonder_application::{
        jev_config::{JEV_REMOTE_ENDPOINT, JevCapability, JevConfig, JevServiceMode},
        jev_runtime::{JevCandidate, JevDecision, JevDecisionRequest, decide},
    };

    let args: Vec<String> = std::env::args().collect();
    assert_eq!(args.len(), 4, "node worker sdk");
    let port = MacosJevPort::new(
        Path::new(&args[1]),
        Path::new(&args[2]),
        Path::new(&args[3]),
        "Yonder",
        "jev",
        Duration::from_secs(30),
    )
    .unwrap();
    let config = JevConfig {
        enabled: true,
        service_mode: JevServiceMode::Remote,
        endpoint: JEV_REMOTE_ENDPOINT.into(),
        capabilities: vec![JevCapability::Cua],
        ..JevConfig::default()
    };
    let request = JevDecisionRequest {
        task_id: "verification-task".into(),
        step_id: "verification-step".into(),
        capability: JevCapability::Cua,
        candidates: vec![
            JevCandidate {
                id: "cua.unavailable".into(),
                dispatchable: false,
                parameter_complete: false,
                action_kind: "bring-to-front".into(),
                target_ref: "verification-app".into(),
                preconditions: vec!["application-ready=true".into()],
                expected_observe: vec!["target-resolved=true".into()],
            },
            JevCandidate {
                id: "handback".into(),
                dispatchable: true,
                parameter_complete: true,
                action_kind: "handback".into(),
                target_ref: "none".into(),
                preconditions: vec![],
                expected_observe: vec![],
            },
        ],
    };
    let started = Instant::now();
    match decide(&config, &port, &request) {
        Ok(JevDecision::Dispatch { .. }) => println!(
            "{{\"remote_call\":true,\"decision\":\"dispatch\",\"elapsed_ms\":{}}}",
            started.elapsed().as_millis()
        ),
        Ok(JevDecision::HandBack { .. }) => println!(
            "{{\"remote_call\":true,\"decision\":\"handback\",\"elapsed_ms\":{}}}",
            started.elapsed().as_millis()
        ),
        Err(error) => {
            eprintln!(
                "{{\"remote_call\":false,\"error\":\"{error:?}\",\"elapsed_ms\":{}}}",
                started.elapsed().as_millis()
            );
            std::process::exit(1);
        }
    }
}

#[cfg(not(target_os = "macos"))]
fn main() {
    eprintln!("capability_unavailable");
    std::process::exit(2);
}
