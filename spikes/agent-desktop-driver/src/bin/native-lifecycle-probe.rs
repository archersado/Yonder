//! Isolated diagnostic entry for comparing two independent WinEvent observers.

#[cfg(target_os = "windows")]
fn main() -> anyhow::Result<()> {
    use std::io::{self, BufRead, Write};

    use anyhow::{Context, ensure};
    use yonder_agent_desktop_driver_spike::host_oracle::{windows, windows_lifecycle};

    let mut args = std::env::args().skip(1);
    let pid: u32 = args
        .next()
        .context("expected fixture PID")?
        .parse()
        .context("invalid fixture PID")?;
    let window_id = args.next().context("expected fixture window ID")?;
    ensure!(
        pid > 0 && args.next().is_none(),
        "expected one fixture target"
    );

    let observer = windows_lifecycle::LifecycleObserver::start()?;
    let initial = windows::capture_window(pid, &window_id)?;
    let mut output = io::stdout().lock();
    writeln!(
        output,
        "{}",
        serde_json::json!({
            "schema_version": "yonder-agent-desktop-lifecycle-probe/v1",
            "phase": "ready",
            "process_start_sha256": initial.process_start_sha256,
            "target_scope_verified": true,
        })
    )?;
    output.flush()?;

    for line in io::stdin().lock().lines() {
        let line = line?;
        ensure!(line.len() <= 256, "diagnostic command too long");
        let command: serde_json::Value = serde_json::from_str(&line)?;
        match command.get("command").and_then(serde_json::Value::as_str) {
            Some("barrier") => {
                let sample = observer.barrier_for(&window_id)?;
                let owner_matches = windows::capture_window(pid, &window_id)
                    .is_ok_and(|current| current == initial);
                writeln!(
                    output,
                    "{}",
                    serde_json::json!({
                        "schema_version": "yonder-agent-desktop-lifecycle-probe/v1",
                        "phase": "barrier",
                        "creates": sample.creates,
                        "destroys": sample.destroys,
                        "marker_observed": sample.marker_observed,
                        "owner_matches_initial": owner_matches,
                        "window_generation_proven": false,
                    })
                )?;
                output.flush()?;
            }
            Some("stop") => break,
            _ => anyhow::bail!("unsupported diagnostic command"),
        }
    }
    Ok(())
}

#[cfg(not(target_os = "windows"))]
fn main() -> anyhow::Result<()> {
    anyhow::bail!("native lifecycle probe currently requires Windows")
}
