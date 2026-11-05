//! CU-S4 read-only native API probe; never dispatches desktop input.

#[cfg(target_os = "windows")]
fn main() -> anyhow::Result<()> {
    use agent_desktop_core::{
        AccessibilityNode, CommandContext, Deadline, IdentifierKind, ObservationOps, TreeOptions,
        WindowFilter, snapshot,
    };
    use sha2::{Digest, Sha256};

    fn collect_fixture_refs<'a>(
        node: &'a AccessibilityNode,
        matches: &mut u32,
        refs: &mut Vec<&'a str>,
    ) {
        if node.identity.native_id.as_ref().is_some_and(|id| {
            id.kind == IdentifierKind::AutomationId && id.value == "YonderProbeButton"
        }) {
            *matches += 1;
            if let Some(reference) = node.ref_id.as_deref() {
                refs.push(reference);
            }
        }
        for child in &node.children {
            collect_fixture_refs(child, matches, refs);
        }
    }

    let mut args = std::env::args().skip(1);
    let pid: u32 = args
        .next()
        .ok_or_else(|| anyhow::anyhow!("expected one PID"))?
        .parse()
        .map_err(|_| anyhow::anyhow!("invalid PID"))?;
    anyhow::ensure!(pid > 0 && args.next().is_none(), "expected one valid PID");
    let home = std::env::var_os("AGENT_DESKTOP_HOME")
        .ok_or_else(|| anyhow::anyhow!("private AGENT_DESKTOP_HOME required"))?;
    anyhow::ensure!(
        std::path::Path::new(&home).is_absolute(),
        "private state path must be absolute"
    );
    anyhow::ensure!(
        std::env::var_os("AGENT_DESKTOP_TRACE").as_deref() == Some(std::ffi::OsStr::new("off")),
        "trace must be off"
    );
    anyhow::ensure!(
        std::env::var_os("AGENT_DESKTOP_SCREENSHOTS").as_deref()
            == Some(std::ffi::OsStr::new("off")),
        "screenshots must be off"
    );

    agent_desktop_windows::ensure_owned_process_mta_and_dpi()
        .map_err(|_| anyhow::anyhow!("Windows COM/DPI bootstrap failed"))?;
    agent_desktop_core::install_private_file_ops(Box::new(
        agent_desktop_windows::WindowsPrivateFile,
    ))
    .map_err(|_| anyhow::anyhow!("private file backend already installed"))?;
    let adapter = agent_desktop_windows::WindowsAdapter::new();
    let windows = adapter
        .list_windows(&WindowFilter::default(), Deadline::after(5_000)?)
        .map_err(|_| anyhow::anyhow!("window enumeration failed"))?;
    let mut targets = windows.into_iter().filter(|window| window.pid.get() == pid);
    let window = targets
        .next()
        .ok_or_else(|| anyhow::anyhow!("target PID has no window"))?;
    anyhow::ensure!(targets.next().is_none(), "target PID has multiple windows");
    anyhow::ensure!(!window.id.is_empty(), "target has no native window ID");
    let host_before =
        yonder_agent_desktop_driver_spike::host_oracle::windows::capture_window(pid, &window.id)?;
    anyhow::ensure!(
        host_before.matches_adapter_window(&window),
        "host and adapter process identity disagree"
    );

    let result = snapshot::run_with_context(
        &adapter,
        &TreeOptions::default(),
        &snapshot::SnapshotTarget {
            app_name: None,
            window_id: Some(&window.id),
        },
        &CommandContext::default(),
        15_000,
    )
    .map_err(|error| {
        let category = match &error {
            agent_desktop_core::AppError::Adapter(_) => "adapter",
            agent_desktop_core::AppError::Io(_) => "io",
            agent_desktop_core::AppError::Json(_) => "json",
            agent_desktop_core::AppError::Internal(_) => "internal",
        };
        anyhow::anyhow!("native snapshot failed: {category}/{}", error.code())
    })?;
    anyhow::ensure!(result.window.pid.get() == pid, "snapshot process changed");
    anyhow::ensure!(result.window.id == window.id, "snapshot window changed");
    let host_after =
        yonder_agent_desktop_driver_spike::host_oracle::windows::capture_window(pid, &window.id)?;
    anyhow::ensure!(
        host_before == host_after && host_after.matches_adapter_window(&result.window),
        "host and snapshot process identity disagree"
    );
    anyhow::ensure!(result.complete, "snapshot is incomplete");
    let snapshot_id = result
        .snapshot_id
        .as_deref()
        .ok_or_else(|| anyhow::anyhow!("snapshot was not persisted"))?;
    let mut selector_matches = 0;
    let mut selector_refs = Vec::new();
    collect_fixture_refs(&result.tree, &mut selector_matches, &mut selector_refs);
    anyhow::ensure!(
        selector_matches == 1 && selector_refs.len() == 1,
        "fixture target is not unique"
    );
    anyhow::ensure!(
        selector_refs[0].starts_with(&format!("@{snapshot_id}:")),
        "selected reference is not snapshot-qualified"
    );

    let digest = |value: &str| format!("{:x}", Sha256::digest(value.as_bytes()));
    println!(
        "{}",
        serde_json::json!({
            "schema_version": "yonder-agent-desktop-native-read-probe/v1",
            "platform": "windows",
            "pid": pid,
            "window_id_sha256": digest(&window.id),
            "process_instance_sha256": window.process_instance.as_deref().map(digest),
            "host_process_start_sha256": host_after.process_start_sha256,
            "host_identity_matches_adapter": true,
            "window_generation_proven": false,
            "snapshot_window_id_sha256": digest(&result.window.id),
            "ref_count": result.refmap.len(),
            "nodes_observed": result.nodes_observed,
            "complete": result.complete,
            "identity_unproven": result.identity_unproven.is_some(),
            "selector_matches": selector_matches,
            "selector_refs": selector_refs.len(),
            "selected_qualified_ref_sha256": digest(selector_refs[0]),
            "snapshot_id_sha256": digest(snapshot_id),
            "root_native_id_kind": result.tree.identity.native_id.as_ref().map(|id| format!("{:?}", id.kind)),
            "root_native_id_sha256": result.tree.identity.native_id.as_ref().map(|id| digest(&id.value)),
        })
    );
    Ok(())
}

#[cfg(not(target_os = "windows"))]
fn main() -> anyhow::Result<()> {
    anyhow::bail!("native read probe currently requires Windows")
}
