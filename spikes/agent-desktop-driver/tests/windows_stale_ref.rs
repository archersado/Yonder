#![cfg(target_os = "windows")]

use std::{
    error::Error,
    io::{BufRead, BufReader, Write},
    process::{Child, Command, Stdio},
};

use agent_desktop_core::{
    AccessibilityNode, CommandContext, Deadline, ErrorCode, IdentifierKind, PrivateFileOps,
    RefStore, TreeOptions, refs::resolve_saved_ref::resolve_saved_ref, snapshot,
};
use tempfile::TempDir;
use yonder_agent_desktop_driver_spike::host_oracle::windows::capture_identity01_window;

struct Fixture(Child);

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

fn button_refs(node: &AccessibilityNode, out: &mut Vec<String>) {
    if node.identity.native_id.as_ref().is_some_and(|id| {
        id.kind == IdentifierKind::AutomationId && id.value == "YonderIdentity01Button"
    }) && let Some(reference) = &node.ref_id
    {
        out.push(reference.clone());
    }
    for child in &node.children {
        button_refs(child, out);
    }
}

#[test]
fn old_ref_is_rejected_after_fixture_rebuild_without_action() -> Result<(), Box<dyn Error>> {
    let home = TempDir::new()?;
    let native_home = home.path().join("native-home");
    agent_desktop_windows::WindowsPrivateFile::new().ensure_private(&native_home)?;
    unsafe {
        std::env::set_var("AGENT_DESKTOP_HOME", &native_home);
        std::env::set_var("AGENT_DESKTOP_TRACE", "off");
        std::env::set_var("AGENT_DESKTOP_SCREENSHOTS", "off");
    }
    agent_desktop_windows::ensure_owned_process_mta_and_dpi()?;
    agent_desktop_core::install_private_file_ops(Box::new(
        agent_desktop_windows::WindowsPrivateFile::new(),
    ))
    .map_err(|_| "private file ops already installed")?;

    let path =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("fixtures/identity01-windows.ps1");
    let mut fixture = Fixture(
        Command::new("powershell.exe")
            .args(["-Sta", "-NoProfile", "-ExecutionPolicy", "Bypass", "-File"])
            .arg(path)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()?,
    );
    let mut output = BufReader::new(fixture.0.stdout.take().ok_or("fixture stdout")?);
    let mut line = String::new();
    output.read_line(&mut line)?;
    let started: serde_json::Value = serde_json::from_str(&line)?;
    let pid = u32::try_from(started["pid"].as_u64().ok_or("fixture pid")?)?;
    let original_id = started["target_window_id"].as_str().ok_or("target HWND")?;
    let decoy_id = started["decoy_window_id"].as_str().ok_or("decoy HWND")?;
    let original = capture_identity01_window(pid, original_id)?;
    let adapter = agent_desktop_windows::WindowsAdapter::new();
    let captured = snapshot::run_with_context(
        &adapter,
        &TreeOptions::default(),
        &snapshot::SnapshotTarget {
            app_name: None,
            window_id: Some(original_id),
        },
        &CommandContext::default(),
        15_000,
    )?;
    assert!(captured.complete && captured.identity_unproven.is_none());
    let mut refs = Vec::new();
    button_refs(&captured.tree, &mut refs);
    assert_eq!(refs.len(), 1);
    let old_ref = RefStore::new()?.load_ref(&refs[0])?;

    fixture
        .0
        .stdin
        .as_mut()
        .ok_or("fixture stdin")?
        .write_all(b"replace-button\n")?;
    line.clear();
    output.read_line(&mut line)?;
    let button_replaced: serde_json::Value = serde_json::from_str(&line)?;
    assert_eq!(button_replaced["window_id"].as_str(), Some(original_id));
    let same_window = capture_identity01_window(pid, original_id)?;
    assert_eq!(
        original.window_token_sha256,
        same_window.window_token_sha256
    );
    let old_element_rebound =
        resolve_saved_ref(&adapter, &old_ref, Deadline::after(10_000)?, None).is_ok();
    assert_eq!(same_window.click_count, 1);
    eprintln!(
        "identity02_read_only button_replaced=true window_generation_unchanged=true old_element_rebound={old_element_rebound}"
    );
    assert!(
        old_element_rebound,
        "fixed upstream no longer rebinds the element"
    );

    fixture
        .0
        .stdin
        .as_mut()
        .ok_or("fixture stdin")?
        .write_all(b"replace\n")?;
    line.clear();
    output.read_line(&mut line)?;
    let replaced: serde_json::Value = serde_json::from_str(&line)?;
    let replacement_id = replaced["replacement_window_id"]
        .as_str()
        .ok_or("replacement HWND")?;
    let replacement = capture_identity01_window(pid, replacement_id)?;
    let decoy = capture_identity01_window(pid, decoy_id)?;
    assert_ne!(
        original.window_token_sha256,
        replacement.window_token_sha256
    );
    assert_eq!((replacement.click_count, decoy.click_count), (1, 1));

    let error = match resolve_saved_ref(&adapter, &old_ref, Deadline::after(10_000)?, None) {
        Ok(_) => return Err("old ref unexpectedly resolved after window replacement".into()),
        Err(error) => error,
    };
    assert!(matches!(
        error.code,
        ErrorCode::StaleRef | ErrorCode::WindowNotFound
    ));
    eprintln!(
        "identity02_read_only old_hwnd_reused={} generation_changed=true old_ref_rejected=true",
        original_id == replacement_id
    );
    let replacement = capture_identity01_window(pid, replacement_id)?;
    let decoy = capture_identity01_window(pid, decoy_id)?;
    assert_eq!((replacement.click_count, decoy.click_count), (1, 1));
    Ok(())
}
