//! Read-only cross-process identity probe for the controlled IDENTITY-01 fixture.

#[cfg(target_os = "windows")]
fn main() -> anyhow::Result<()> {
    use anyhow::{Context, ensure};
    use yonder_agent_desktop_driver_spike::host_oracle::windows::capture_identity01_window;

    let mut args = std::env::args().skip(1);
    let pid: u32 = args.next().context("fixture PID required")?.parse()?;
    let target_id = args.next().context("target HWND required")?;
    let decoy_id = args.next().context("decoy HWND required")?;
    ensure!(args.next().is_none(), "unexpected diagnostic argument");
    let target = capture_identity01_window(pid, &target_id)?;
    let decoy = capture_identity01_window(pid, &decoy_id)?;
    ensure!(target.role == 1 && decoy.role == 2, "fixture role mismatch");
    ensure!(
        target.window.process_start_sha256 == decoy.window.process_start_sha256,
        "fixture process mismatch"
    );
    ensure!(
        target.window_token_sha256 != decoy.window_token_sha256,
        "fixture generation token collision"
    );
    println!(
        "{}",
        serde_json::json!({
            "schema_version": "yonder-agent-desktop-identity01-read-probe/v1",
            "platform": "windows",
            "target_scope_verified": true,
            "decoy_scope_verified": true,
            "roles_distinct": true,
            "tokens_distinct": true,
            "target_click_count_encoded": target.click_count,
            "decoy_click_count_encoded": decoy.click_count,
            "target_clicks_zero": target.click_count == 1,
            "decoy_clicks_zero": decoy.click_count == 1,
            "window_generation_proven": false,
            "worker_action_path_proven": false,
        })
    );
    ensure!(
        target.click_count == 1 && decoy.click_count == 1,
        "fixture pre-action state changed"
    );
    Ok(())
}

#[cfg(not(target_os = "windows"))]
fn main() -> anyhow::Result<()> {
    anyhow::bail!("IDENTITY-01 read probe requires Windows")
}
