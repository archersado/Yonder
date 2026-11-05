//! Read-only snapshot for the controlled IDENTITY-01 Windows fixture.

use agent_desktop_core::{
    AccessibilityNode, CommandContext, Deadline, IdentifierKind, ObservationOps, TreeOptions,
    WindowFilter, snapshot,
};

use crate::{
    backend::SnapshotReply,
    clock::monotonic_millis,
    host_oracle::{windows::capture_identity01_window, windows_lifecycle::LifecycleObserver},
    protocol::{ErrorClass, HostRequest, IdentityVerdict, Status},
};

fn rejected(error_class: ErrorClass) -> SnapshotReply {
    SnapshotReply {
        status: Status::Rejected,
        error_class,
        snapshot_id: String::new(),
        qualified_refs: Vec::new(),
        identity: IdentityVerdict::Unavailable,
    }
}

fn collect_refs<'a>(node: &'a AccessibilityNode, matches: &mut usize, refs: &mut Vec<&'a str>) {
    if node.identity.native_id.as_ref().is_some_and(|id| {
        id.kind == IdentifierKind::AutomationId && id.value == "YonderIdentity01Button"
    }) {
        *matches += 1;
        if let Some(reference) = node.ref_id.as_deref() {
            refs.push(reference);
        }
    }
    for child in &node.children {
        collect_refs(child, matches, refs);
    }
}

pub(super) fn snapshot_identity01(
    adapter: &agent_desktop_windows::WindowsAdapter,
    request: &HostRequest,
) -> SnapshotReply {
    let HostRequest::Snapshot {
        sample_id,
        target_identity,
        deadline_mono_ms,
        ..
    } = request
    else {
        return rejected(ErrorClass::Schema);
    };
    if sample_id != "IDENTITY-01"
        || target_identity.platform != "windows"
        || target_identity.fixture_id != "identity-01"
        || target_identity.topology_generation != 1
    {
        return rejected(ErrorClass::Unsupported);
    }
    let Ok(before) = capture_identity01_window(
        target_identity.process_id,
        &target_identity.native_window_id,
    ) else {
        return rejected(ErrorClass::Identity);
    };
    if before.role != 1
        || before.click_count != 1
        || before.window.process_start_sha256 != target_identity.process_start_sha256
        || before.window_token_sha256 != target_identity.window_token_sha256
    {
        return rejected(ErrorClass::Identity);
    }
    let Ok(lifecycle) = LifecycleObserver::start() else {
        return rejected(ErrorClass::Identity);
    };
    let Ok(barrier_before) = lifecycle.barrier_for(&target_identity.native_window_id) else {
        return rejected(ErrorClass::Identity);
    };
    let Ok(now) = monotonic_millis() else {
        return rejected(ErrorClass::Deadline);
    };
    let remaining = deadline_mono_ms.saturating_sub(now).min(15_000);
    if remaining == 0 {
        return rejected(ErrorClass::Deadline);
    }
    let Ok(deadline) = Deadline::after(remaining) else {
        return rejected(ErrorClass::Deadline);
    };
    let Ok(windows) = adapter.list_windows(&WindowFilter::default(), deadline) else {
        eprintln!("identity01_snapshot_stage=list_windows");
        return rejected(ErrorClass::Driver);
    };
    let mut targets = windows.into_iter().filter(|window| {
        window.pid.get() == target_identity.process_id
            && window.id == target_identity.native_window_id
            && before.window.matches_adapter_window(window)
    });
    if targets.next().is_none() || targets.next().is_some() {
        return rejected(ErrorClass::Identity);
    }
    let result = match snapshot::run_with_context(
        adapter,
        &TreeOptions::default(),
        &snapshot::SnapshotTarget {
            app_name: None,
            window_id: Some(&target_identity.native_window_id),
        },
        &CommandContext::default(),
        remaining,
    ) {
        Ok(result) => result,
        Err(error) => {
            let category = match &error {
                agent_desktop_core::AppError::Adapter(_) => "adapter",
                agent_desktop_core::AppError::Io(_) => "io",
                agent_desktop_core::AppError::Json(_) => "json",
                agent_desktop_core::AppError::Internal(_) => "internal",
            };
            eprintln!(
                "identity01_snapshot_stage=snapshot category={category} code={}",
                error.code()
            );
            return rejected(ErrorClass::Driver);
        }
    };
    let Ok(after) = capture_identity01_window(
        target_identity.process_id,
        &target_identity.native_window_id,
    ) else {
        return rejected(ErrorClass::Stale);
    };
    let Ok(barrier_after) = lifecycle.barrier_for(&target_identity.native_window_id) else {
        return rejected(ErrorClass::Identity);
    };
    if before != after
        || barrier_before != barrier_after
        || !after.window.matches_adapter_window(&result.window)
        || !result.complete
        || result.identity_unproven.is_some()
    {
        return rejected(ErrorClass::Identity);
    }
    let Some(snapshot_id) = result.snapshot_id else {
        eprintln!("identity01_snapshot_stage=persisted_snapshot");
        return rejected(ErrorClass::Driver);
    };
    let mut matches = 0;
    let mut refs = Vec::new();
    collect_refs(&result.tree, &mut matches, &mut refs);
    if matches != 1 || refs.len() != 1 {
        return rejected(ErrorClass::Ambiguous);
    }
    if !refs[0].starts_with(&format!("@{snapshot_id}:")) {
        return rejected(ErrorClass::Schema);
    }
    SnapshotReply {
        status: Status::Ok,
        error_class: ErrorClass::None,
        snapshot_id,
        qualified_refs: vec![refs[0].to_owned()],
        identity: IdentityVerdict::Match,
    }
}
