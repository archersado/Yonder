#[cfg(target_os = "macos")]
use security_framework::passwords::{get_generic_password, set_generic_password};
use serde::{Deserialize, Serialize};
use std::{
    fs,
    io::{BufRead, BufReader, Write},
    path::{Path, PathBuf},
    process::{Command, Stdio},
    sync::mpsc,
    thread,
    time::{Duration, Instant},
};
use yonder_application::{
    jev_config::JevConfig,
    jev_runtime::{JevDecisionError, JevDecisionPort, JevDecisionRequest, JevModelChoice},
};

const MAX_JEV_WORKER_TIMEOUT: Duration = Duration::from_secs(30);

pub struct MacosJevPort {
    node: PathBuf,
    script: PathBuf,
    sdk: PathBuf,
    service: String,
    account: String,
    timeout: Duration,
}

#[derive(Serialize)]
struct WorkerRequest<'a> {
    endpoint: &'a str,
    api_key: &'a str,
    timeout_ms: u64,
    candidates: &'a [yonder_application::jev_runtime::JevCandidate],
}

#[derive(Deserialize)]
struct WorkerResponse {
    candidate_id: String,
    confidence: f32,
}

impl MacosJevPort {
    pub fn new(
        node: &Path,
        script: &Path,
        sdk: &Path,
        service: &str,
        account: &str,
        timeout: Duration,
    ) -> Result<Self, JevDecisionError> {
        if timeout.is_zero()
            || timeout > MAX_JEV_WORKER_TIMEOUT
            || [node, script, sdk]
                .iter()
                .any(|path| !path.is_absolute() || !path.is_file())
            || service.trim().is_empty()
            || account.trim().is_empty()
        {
            return Err(JevDecisionError::InvalidInput);
        }
        let package = sdk
            .parent()
            .and_then(Path::parent)
            .map(|path| path.join("package.json"))
            .ok_or(JevDecisionError::DependencyUnavailable)?;
        if !package.is_file() {
            return Err(JevDecisionError::DependencyUnavailable);
        }
        let version = fs::read_to_string(package)
            .ok()
            .and_then(|value| serde_json::from_str::<serde_json::Value>(&value).ok())
            .and_then(|value| {
                value
                    .get("version")
                    .and_then(|value| value.as_str())
                    .map(str::to_owned)
            });
        if version.as_deref() != Some("0.6.0") {
            return Err(JevDecisionError::DependencyUnavailable);
        }
        Ok(Self {
            node: node.into(),
            script: script.into(),
            sdk: sdk.into(),
            service: service.into(),
            account: account.into(),
            timeout,
        })
    }

    fn credential(&self) -> Result<String, JevDecisionError> {
        #[cfg(target_os = "macos")]
        {
            let key = get_generic_password(&self.service, &self.account)
                .map_err(|_| JevDecisionError::CredentialUnavailable)?;
            let key =
                String::from_utf8(key).map_err(|_| JevDecisionError::CredentialUnavailable)?;
            if !valid_credential(&key) {
                return Err(JevDecisionError::CredentialUnavailable);
            }
            Ok(key)
        }
        #[cfg(not(target_os = "macos"))]
        {
            Err(JevDecisionError::DependencyUnavailable)
        }
    }

    fn run_worker(&self, request: &WorkerRequest<'_>, timeout: Duration) -> Result<WorkerResponse, JevDecisionError> {
        let payload =
            serde_json::to_string(request).map_err(|_| JevDecisionError::InvalidResponse)?;
        let mut child = Command::new(&self.node)
            .arg(&self.script)
            .arg(&self.sdk)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|_| JevDecisionError::DependencyUnavailable)?;
        let Some(mut stdin) = child.stdin.take() else {
            return Err(JevDecisionError::WorkerFailed);
        };
        let stdout = child.stdout.take().ok_or(JevDecisionError::WorkerFailed)?;
        let (sender, receiver) = mpsc::channel();
        thread::spawn(move || {
            let mut reader = BufReader::new(stdout);
            let mut line = Vec::new();
            match reader.read_until(b'\n', &mut line) {
                Ok(0) | Err(_) => {}
                Ok(_) if line.len() <= 65_536 => {
                    let _ = sender.send(Ok(line));
                }
                _ => {
                    let _ = sender.send(Err(()));
                }
            }
        });
        stdin
            .write_all(payload.as_bytes())
            .map_err(|_| JevDecisionError::WorkerFailed)?;
        drop(stdin);

        let started = Instant::now();
        loop {
            match child.try_wait() {
                Ok(Some(status)) if status.success() => break,
                Ok(Some(_)) => return Err(JevDecisionError::RemoteError),
                Ok(None) if started.elapsed() < timeout => {
                    thread::sleep(Duration::from_millis(10))
                }
                _ => {
                    let _ = child.kill();
                    let _ = child.wait();
                    return Err(JevDecisionError::TimedOut);
                }
            }
        }
        let output = receiver
            .recv_timeout(Duration::from_millis(100))
            .map_err(|_| JevDecisionError::InvalidResponse)?
            .map_err(|_| JevDecisionError::InvalidResponse)?;
        serde_json::from_slice(&output).map_err(|_| JevDecisionError::InvalidResponse)
    }
}

pub fn jev_credential_configured(service: &str, account: &str) -> bool {
    #[cfg(target_os = "macos")]
    {
        get_generic_password(service, account)
            .ok()
            .is_some_and(|key| String::from_utf8(key).is_ok_and(|key| valid_credential(&key)))
    }
    #[cfg(not(target_os = "macos"))]
    {
        let _ = (service, account);
        false
    }
}

pub fn set_jev_credential(
    service: &str,
    account: &str,
    api_key: &str,
) -> Result<(), JevDecisionError> {
    if !valid_credential(api_key) {
        return Err(JevDecisionError::InvalidInput);
    }
    #[cfg(target_os = "macos")]
    {
        set_generic_password(service, account, api_key.as_bytes())
            .map_err(|_| JevDecisionError::CredentialUnavailable)
    }
    #[cfg(not(target_os = "macos"))]
    {
        let _ = (service, account);
        Err(JevDecisionError::DependencyUnavailable)
    }
}

fn valid_credential(api_key: &str) -> bool {
    !api_key.trim().is_empty() && api_key.len() <= 8_192 && !api_key.contains('\0')
}

impl JevDecisionPort for MacosJevPort {
    fn choose(
        &self,
        config: &JevConfig,
        request: &JevDecisionRequest,
    ) -> Result<JevModelChoice, JevDecisionError> {
        let api_key = self.credential()?;
        // 面板控制单次选择预算；Adapter 再以硬上限保证快脑不会长期占用
        // Gateway 执行槽位。两者均有界，且不进行自动重试。
        let timeout = Duration::from_millis(config.time_limit_ms).min(self.timeout);
        let worker_request = WorkerRequest {
            endpoint: &config.endpoint,
            api_key: &api_key,
            timeout_ms: timeout.as_millis() as u64,
            candidates: &request.candidates,
        };
        let response = self.run_worker(&worker_request, timeout)?;
        Ok(JevModelChoice {
            candidate_id: response.candidate_id,
            confidence: response.confidence,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::OnceLock;
    use yonder_application::jev_runtime::JevCandidate;

    fn node_path() -> PathBuf {
        let output = Command::new("/usr/bin/which").arg("node").output().unwrap();
        PathBuf::from(String::from_utf8_lossy(&output.stdout).trim())
    }

    fn sdk_path() -> PathBuf {
        static SDK: OnceLock<PathBuf> = OnceLock::new();
        SDK.get_or_init(|| {
            let package = std::env::temp_dir()
                .join(format!("yonder-jev-sdk-{}", std::process::id()))
                .join("@typesafe-ai/sdk");
            fs::create_dir_all(package.join("dist")).unwrap();
            fs::write(package.join("package.json"), r#"{"version":"0.6.0"}"#).unwrap();
            fs::write(package.join("dist/index.mjs"), "export default {};\n").unwrap();
            fs::canonicalize(package.join("dist/index.mjs")).unwrap()
        })
        .clone()
    }

    #[test]
    fn rejects_missing_keychain_credential() {
        let node = node_path();
        let script =
            fs::canonicalize(Path::new(env!("CARGO_MANIFEST_DIR")).join("src/jev_worker.mjs"))
                .unwrap();
        let port = MacosJevPort::new(
            &node,
            &script,
            &sdk_path(),
            "Yonder-Test-Does-Not-Exist",
            "nobody",
            Duration::from_millis(1500),
        )
        .unwrap();
        assert_eq!(
            port.credential(),
            Err(JevDecisionError::CredentialUnavailable)
        );
    }

    #[test]
    fn validates_credential_before_keychain_write() {
        assert!(!valid_credential(""));
        assert!(!valid_credential("   "));
        assert!(!valid_credential("a\0b"));
        assert!(valid_credential("jev-test-key"));
    }

    #[test]
    fn runs_worker_with_bounded_candidates() {
        let script =
            std::env::temp_dir().join(format!("yonder-jev-worker-{}.mjs", std::process::id()));
        fs::write(
            &script,
            "process.stdout.write(JSON.stringify({candidate_id:'handback',confidence:1})+'\\n');\n",
        )
        .unwrap();
        let node = node_path();
        let port = MacosJevPort::new(
            &node,
            &script,
            &sdk_path(),
            "Yonder",
            "jev",
            Duration::from_millis(3000),
        )
        .unwrap();
        let request = WorkerRequest {
            endpoint: "https://api.typesafe.ai",
            api_key: "test-only",
            timeout_ms: 3000,
            candidates: &[
                JevCandidate {
                    id: "cua.click".into(),
                    dispatchable: true,
                    parameter_complete: true,
                    action_kind: "bring-to-front".into(), target_ref: "app-1".into(), preconditions: vec!["application-ready=true".into()], expected_observe: vec!["target-resolved=true".into()],
                },
                JevCandidate {
                    id: "handback".into(),
                    dispatchable: true,
                    parameter_complete: true,
                    action_kind: "handback".into(), target_ref: "none".into(), preconditions: vec![], expected_observe: vec![],
                },
            ],
        };
        let result = port.run_worker(&request, Duration::from_millis(3000)).unwrap();
        assert_eq!(result.candidate_id, "handback");
        assert!((0.0..=1.0).contains(&result.confidence));
        let _ = fs::remove_file(script);
    }

    #[test]
    fn rejects_timeout_above_product_limit() {
        let node = node_path();
        let script =
            fs::canonicalize(Path::new(env!("CARGO_MANIFEST_DIR")).join("src/jev_worker.mjs"))
                .unwrap();
        assert!(matches!(
            MacosJevPort::new(
                &node,
                &script,
                &sdk_path(),
                "Yonder",
                "jev",
                MAX_JEV_WORKER_TIMEOUT + Duration::from_millis(1)
            ),
            Err(JevDecisionError::InvalidInput)
        ));
    }
}
