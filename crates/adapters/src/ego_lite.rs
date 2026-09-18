use serde::{Deserialize, Serialize};
use std::{fs::OpenOptions, io::Write, os::unix::fs::OpenOptionsExt, path::{Path, PathBuf}, process::{Command, Stdio}, sync::atomic::{AtomicU64, Ordering}, time::{Duration, Instant}};
use yonder_application::{ExecutionAttempt, browser_use::{BrowserAction, BrowserOutcome, BrowserTaskRef, BrowserUsePort, valid_ref}, computer_use::UnknownReason};

pub struct EgoLiteBridge { cli: PathBuf, timeout: Duration }
static SCRIPT_SEQUENCE: AtomicU64 = AtomicU64::new(1);

#[derive(Serialize)]
struct Request<'a> { operation: &'a str, name: Option<&'a str>, external_task_ref: Option<&'a str> }

#[derive(Deserialize)]
struct Response { operation: String, external_task_ref: String, ownership: Option<String>, managed_pages: Option<usize>, finished: Option<bool> }

impl EgoLiteBridge {
    pub fn new(cli: &Path, timeout: Duration) -> Result<Self, UnknownReason> {
        if !cli.is_absolute() || !cli.is_file() || timeout.is_zero() || timeout > Duration::from_secs(120) { return Err(UnknownReason::DependencyUnavailable); }
        Ok(Self { cli: cli.into(), timeout })
    }

    fn run(&self, request: Request<'_>) -> BrowserOutcome {
        let expected_ref = request.external_task_ref.map(str::to_owned);
        let operation = request.operation.to_owned();
        let json = match serde_json::to_string(&request) { Ok(value) => value, Err(_) => return BrowserOutcome::Unknown(UnknownReason::InvalidInput) };
        let quoted = match serde_json::to_string(&json) { Ok(value) => value, Err(_) => return BrowserOutcome::Unknown(UnknownReason::InvalidInput) };
        let sequence = SCRIPT_SEQUENCE.fetch_add(1,Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!("yonda-ego-sdk-{}-{sequence}.mjs",std::process::id()));
        let response_path = std::env::temp_dir().join(format!("yonda-ego-result-{}-{sequence}.json",std::process::id()));
        let response_quoted = match serde_json::to_string(response_path.to_string_lossy().as_ref()) { Ok(value) => value, Err(_) => return BrowserOutcome::Unknown(UnknownReason::InvalidInput) };
        let script = format!("const request = JSON.parse({quoted});\nconst responsePath = {response_quoted};\nconst fs = await import('node:fs/promises');\n{}",include_str!("ego_lite_worker.mjs"));
        if OpenOptions::new().write(true).create_new(true).mode(0o600).open(&response_path).is_err() { return BrowserOutcome::Unknown(UnknownReason::WorkerFailed); }
        let script_input = (|| {
            let mut file = OpenOptions::new().write(true).create_new(true).mode(0o600).open(&path).ok()?;
            file.write_all(script.as_bytes()).ok()?;
            drop(file);
            OpenOptions::new().read(true).open(&path).ok()
        })();
        let Some(script_input) = script_input else { let _ = std::fs::remove_file(&path); let _ = std::fs::remove_file(&response_path); return BrowserOutcome::Unknown(UnknownReason::WorkerFailed) };
        let mut child = match Command::new(&self.cli).arg("nodejs")
            .stdin(Stdio::from(script_input)).stdout(Stdio::null()).stderr(Stdio::null()).spawn() {
            Ok(child) => { let _ = std::fs::remove_file(&path); child },
            Err(_) => { let _ = std::fs::remove_file(&path); let _ = std::fs::remove_file(&response_path); return BrowserOutcome::Unknown(UnknownReason::DependencyUnavailable) },
        };
        let started = Instant::now();
        let status = loop {
            match child.try_wait() {
                Ok(Some(status)) => break Some(status),
                Ok(None) if started.elapsed() < self.timeout => std::thread::sleep(Duration::from_millis(10)),
                _ => { let _ = child.kill(); let _ = child.wait(); break None },
            }
        };
        let outcome = if status.is_none() { BrowserOutcome::Unknown(UnknownReason::TimedOut) }
            else if !status.is_some_and(|status| status.success()) { BrowserOutcome::Unknown(UnknownReason::WorkerFailed) }
            else { match std::fs::read(&response_path) {
                Ok(output) if output.len() <= 65_536 => classify(&operation, expected_ref.as_deref(), &output),
                _ => BrowserOutcome::Unknown(UnknownReason::InvalidResponse),
            }};
        let _ = std::fs::remove_file(&response_path);
        outcome
    }
}

impl BrowserUsePort for EgoLiteBridge {
    fn dispatch(&self, _: &ExecutionAttempt, action: &BrowserAction) -> BrowserOutcome {
        match action {
            BrowserAction::Create { name } if !name.trim().is_empty() && name.len() <= 200 && !name.contains('\0') => self.run(Request { operation: "create", name: Some(name), external_task_ref: None }),
            BrowserAction::Observe { external_task_ref } if valid_ref(external_task_ref) => self.run(Request { operation: "observe", name: None, external_task_ref: Some(external_task_ref) }),
            BrowserAction::HandOff { external_task_ref } if valid_ref(external_task_ref) => self.run(Request { operation: "hand_off", name: None, external_task_ref: Some(external_task_ref) }),
            BrowserAction::TakeOver { external_task_ref } if valid_ref(external_task_ref) => self.run(Request { operation: "take_over", name: None, external_task_ref: Some(external_task_ref) }),
            BrowserAction::Finish { external_task_ref } if valid_ref(external_task_ref) => self.run(Request { operation: "finish", name: None, external_task_ref: Some(external_task_ref) }),
            _ => BrowserOutcome::Unknown(UnknownReason::InvalidInput),
        }
    }
}

fn classify(operation: &str, expected_ref: Option<&str>, output: &[u8]) -> BrowserOutcome {
    let response: Response = match serde_json::from_slice(output) { Ok(value) => value, Err(_) => return BrowserOutcome::Unknown(UnknownReason::InvalidResponse) };
    if response.operation != operation || !valid_ref(&response.external_task_ref) || expected_ref.is_some_and(|value| value != response.external_task_ref) {
        return BrowserOutcome::Unknown(UnknownReason::IdentityMismatch);
    }
    if operation == "finish" {
        return if response.finished == Some(true) { BrowserOutcome::Finished { external_task_ref: response.external_task_ref } } else { BrowserOutcome::Unknown(UnknownReason::ObserveFailed) };
    }
    match (response.ownership, response.managed_pages) {
        (Some(ownership), Some(managed_pages)) if !ownership.is_empty() => BrowserOutcome::Observed(BrowserTaskRef { external_task_ref: response.external_task_ref, ownership, managed_pages }),
        _ => BrowserOutcome::Unknown(UnknownReason::ObserveFailed),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn response_must_match_operation_and_space() {
        let ok = br#"{"operation":"observe","external_task_ref":"ego:7","ownership":"agent","managed_pages":1}"#;
        assert!(matches!(classify("observe", Some("ego:7"), ok), BrowserOutcome::Observed(_)));
        assert_eq!(classify("observe", Some("ego:8"), ok), BrowserOutcome::Unknown(UnknownReason::IdentityMismatch));
        assert_eq!(classify("hand_off", Some("ego:7"), ok), BrowserOutcome::Unknown(UnknownReason::IdentityMismatch));
        assert_eq!(classify("finish", Some("ego:7"), br#"{"operation":"finish","external_task_ref":"ego:7","finished":true}"#), BrowserOutcome::Finished { external_task_ref: "ego:7".into() });
    }
}
