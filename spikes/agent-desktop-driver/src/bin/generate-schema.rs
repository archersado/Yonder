use std::{
    env, fs,
    path::{Path, PathBuf},
    process::Command,
};

use anyhow::{Context, Result, bail};
use schemars::schema_for;
use yonder_agent_desktop_driver_spike::{
    SOURCE_REVISION,
    evidence::{
        Evidence, VerificationInputs, definition_sha256, file_sha256, verify_catalog_inputs,
        verify_file,
    },
    protocol::{HostRequest, WorkerResponse},
};

fn main() -> Result<()> {
    let check = env::args().skip(1).any(|arg| arg == "--check");
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let openspec = manifest.join("../../openspec/changes/cu-s4-agent-desktop-driver-spike");
    verify_catalog_inputs(
        &openspec.join("action-allowlist-v1.json"),
        &openspec.join("sample-definitions-v1.json"),
        &openspec.join("sample-catalog.md"),
    )?;
    let fixtures = openspec.join("schema-fixtures");
    let inputs = VerificationInputs {
        allowlist: openspec.join("action-allowlist-v1.json"),
        definitions: openspec.join("sample-definitions-v1.json"),
        build_manifest: manifest.join("build-manifest.json"),
        bridge_contract: openspec.join("bridge-contract-v1.md"),
        artifact: fixtures.join("fixture-artifact.txt"),
        contract_fixture_artifact: fixtures.join("fixture-artifact.txt"),
        contract_fixture_provenance: fixtures.join("fixture-provenance.json"),
        distribution: None,
    };
    if !check {
        refresh_fixtures(&fixtures, &inputs)?;
    }
    verify_file(&fixtures.join("observed-pass.json"), &inputs)?;
    for name in [
        "invalid-element-action-pass.json",
        "invalid-element-unknown-pass.json",
        "invalid-observe-pass.json",
        "invalid-error-class-pass.json",
        "invalid-hash-pass.json",
    ] {
        if verify_file(&fixtures.join(name), &inputs).is_ok() {
            bail!("无效 fixture 被接受: {name}");
        }
    }
    let bridge = serde_json::to_vec_pretty(&schema_for!((HostRequest, WorkerResponse)))?;
    let evidence = serde_json::to_vec_pretty(&schema_for!(Evidence))?;
    sync(
        &manifest.join("schemas/bridge-schema-v1.json"),
        &bridge,
        check,
    )?;
    sync(
        &manifest.join("schemas/evidence-schema-v1.json"),
        &evidence,
        check,
    )?;
    sync(&openspec.join("evidence-schema-v1.json"), &evidence, check)?;
    verify_build_manifest(&manifest, &openspec)?;
    Ok(())
}

fn verify_build_manifest(manifest_root: &Path, openspec: &Path) -> Result<()> {
    let value: serde_json::Value =
        serde_json::from_slice(&fs::read(manifest_root.join("build-manifest.json"))?)?;
    let checks = [
        ("source_revision", SOURCE_REVISION.to_owned()),
        (
            "cargo_lock_sha256",
            file_sha256(&manifest_root.join("Cargo.lock"))?,
        ),
        (
            "bridge_contract_sha256",
            file_sha256(&openspec.join("bridge-contract-v1.md"))?,
        ),
        (
            "action_allowlist_sha256",
            file_sha256(&openspec.join("action-allowlist-v1.json"))?,
        ),
        (
            "sample_definitions_sha256",
            file_sha256(&openspec.join("sample-definitions-v1.json"))?,
        ),
        (
            "generated_bridge_schema_sha256",
            file_sha256(&manifest_root.join("schemas/bridge-schema-v1.json"))?,
        ),
        (
            "generated_evidence_schema_sha256",
            file_sha256(&manifest_root.join("schemas/evidence-schema-v1.json"))?,
        ),
        (
            "authoritative_evidence_schema_sha256",
            file_sha256(&openspec.join("evidence-schema-v1.json"))?,
        ),
    ];
    for (field, expected) in checks {
        if value[field].as_str() != Some(&expected) {
            bail!("build manifest 字段漂移: {field}");
        }
    }
    let repo = manifest_root.join("../..");
    let tree_hash = harness_tree_sha256(manifest_root)?;
    if value["yonder_harness_tree_sha256"].as_str() != Some(&tree_hash) {
        bail!("build manifest 未绑定实际 harness tree");
    }
    let dirty_hash = match Command::new("git")
        .args(["status", "--porcelain", "--", "spikes/agent-desktop-driver"])
        .current_dir(&repo)
        .output()
    {
        Ok(output) if output.status.success() => {
            yonder_agent_desktop_driver_spike::evidence::sha256(&output.stdout)
        }
        _ => env::var("YONDER_GIT_DIRTY_STATUS_SHA256")
            .context("git 不可用时必须显式提供 YONDER_GIT_DIRTY_STATUS_SHA256")?,
    };
    if value["git_dirty_status_sha256"].as_str() != Some(&dirty_hash) {
        bail!("build manifest 未绑定实际 dirty status");
    }
    let planned = value["planned_targets"]
        .as_array()
        .context("planned_targets")?;
    let verified = value["verified_targets"]
        .as_array()
        .context("verified_targets")?;
    if planned.len() != 4
        || verified.len() != 1
        || verified[0] != "x86_64-pc-windows-msvc"
        || value["native_build_status"] != "contract_only_native_backend_unsupported"
    {
        bail!("target 验证状态不得把计划目标冒充已构建目标");
    }
    if value["windows_crt"]
        .as_str()
        .is_none_or(|v| !v.contains("crt-static"))
        || value["build_commands"]
            .as_array()
            .is_none_or(|v| v.len() != 4)
        || value["verification_commands"]
            .as_array()
            .is_none_or(|v| v.len() < 4)
        || value["artifacts"].as_array().is_none_or(|v| !v.is_empty())
    {
        bail!("CRT、构建命令或 artifact provenance 不一致");
    }
    Ok(())
}

fn harness_tree_sha256(root: &Path) -> Result<String> {
    fn collect(dir: &Path, root: &Path, files: &mut Vec<PathBuf>) -> Result<()> {
        for entry in fs::read_dir(dir)? {
            let path = entry?.path();
            if path
                .file_name()
                .and_then(|v| v.to_str())
                .is_some_and(|v| v.starts_with("target"))
            {
                continue;
            }
            if path.is_dir() {
                collect(&path, root, files)?;
            } else if path != root.join("build-manifest.json") {
                files.push(path);
            }
        }
        Ok(())
    }
    let mut files = Vec::new();
    collect(root, root, &mut files)?;
    files.sort();
    let mut bytes = Vec::new();
    for path in files {
        let relative = path
            .strip_prefix(root)?
            .to_string_lossy()
            .replace('\\', "/");
        bytes.extend(relative.as_bytes());
        bytes.push(0);
        bytes.extend(fs::read(path)?);
        bytes.push(0);
    }
    Ok(yonder_agent_desktop_driver_spike::evidence::sha256(&bytes))
}

fn refresh_fixtures(fixtures: &Path, inputs: &VerificationInputs) -> Result<()> {
    for entry in fs::read_dir(fixtures)? {
        let path = entry?.path();
        if path.extension().and_then(|value| value.to_str()) != Some("json") {
            continue;
        }
        let mut value: serde_json::Value = serde_json::from_slice(&fs::read(&path)?)?;
        if value.get("predicate_id").is_none() {
            continue;
        }
        let predicate = value["predicate_id"]
            .as_str()
            .context("fixture predicate_id")?
            .to_owned();
        let oracle = value["oracle_id"]
            .as_str()
            .context("fixture oracle_id")?
            .to_owned();
        value["build_manifest_sha256"] = file_sha256(&inputs.build_manifest)?.into();
        value["bridge_contract_sha256"] = file_sha256(&inputs.bridge_contract)?.into();
        value["artifact_sha256"] = file_sha256(&inputs.artifact)?.into();
        value["predicate_sha256"] =
            definition_sha256(&inputs.definitions, "predicates", &predicate)?.into();
        value["oracle_sha256"] = definition_sha256(&inputs.definitions, "oracles", &oracle)?.into();
        if value.get("error_class").is_none() {
            value["error_class"] = "none".into();
        }
        value["evidence_scope"] = "contract_only".into();
        if value.get("stable_observation_count").is_none() {
            let observed = value["action_result"] == "observed";
            value["stable_observation_count"] = if observed { 2.into() } else { 0.into() };
            value["stable_observation_interval_ms"] = if observed { 250.into() } else { 0.into() };
            value["host_oracle_independent"] = observed.into();
            value["cleanup_probes"] = if value["cleanup_verdict"] == "released" {
                serde_json::json!({
                    "process_tree_gone": true,
                    "desktop_handles_gone": true,
                    "input_hooks_gone": true,
                    "permissions_released": true,
                    "temp_state_removed": true
                })
            } else {
                serde_json::Value::Null
            };
            value["distribution"] = serde_json::Value::Null;
        }
        if path.file_name().and_then(|value| value.to_str()) == Some("invalid-hash-pass.json") {
            value["artifact_sha256"] =
                "0000000000000000000000000000000000000000000000000000000000000000".into();
        }
        let mut encoded = serde_json::to_vec_pretty(&value)?;
        encoded.push(b'\n');
        fs::write(path, encoded)?;
    }
    Ok(())
}

fn sync(path: &Path, content: &[u8], check: bool) -> Result<()> {
    let mut expected = content.to_vec();
    expected.push(b'\n');
    if check {
        if fs::read(path).with_context(|| format!("读取 {}", path.display()))? != expected {
            bail!("生成文件已漂移: {}", path.display());
        }
    } else {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(path, expected)?;
    }
    Ok(())
}
