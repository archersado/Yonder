//! Read-only Windows hook readiness and aggregate input generation probe.

#[cfg(target_os = "windows")]
fn main() -> anyhow::Result<()> {
    use anyhow::{Context, ensure};
    use std::{thread, time::Duration};
    use yonder_agent_desktop_driver_spike::host_oracle::windows_input::InputHookDiagnostic;

    let mut args = std::env::args().skip(1);
    let duration_ms: u64 = args.next().context("duration_ms required")?.parse()?;
    ensure!(
        (100..=30_000).contains(&duration_ms) && args.next().is_none(),
        "duration_ms must be 100..30000"
    );
    let observer = InputHookDiagnostic::start()?;
    let before = observer.sample();
    ensure!(
        before.diagnostic_thread_running && !before.thread_or_desktop_lost,
        "input diagnostic thread unavailable"
    );
    thread::sleep(Duration::from_millis(duration_ms));
    let after = observer.sample();
    println!(
        "{}",
        serde_json::json!({
            "schema_version": "yonder-agent-desktop-input-observer-probe/v1",
            "platform": "windows",
            "diagnostic_thread_running_before": before.diagnostic_thread_running,
            "diagnostic_thread_running_after": after.diagnostic_thread_running,
            "thread_or_desktop_lost": after.thread_or_desktop_lost,
            "admission_ready": after.admission_ready,
            "hook_health_proven": false,
            "generation_non_decreasing": after.generation >= before.generation,
            "user_events_observed": after.user_count.saturating_sub(before.user_count),
            "unknown_injected_events_observed": after.unknown_count.saturating_sub(before.unknown_count),
            "driver_attribution_proven": false,
            "human_input_sample_proven": false,
        })
    );
    ensure!(
        after.diagnostic_thread_running && !after.thread_or_desktop_lost,
        "input diagnostic thread lost"
    );
    Ok(())
}

#[cfg(not(target_os = "windows"))]
fn main() -> anyhow::Result<()> {
    anyhow::bail!("native input observer probe requires Windows")
}
