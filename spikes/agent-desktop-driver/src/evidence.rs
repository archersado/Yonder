use std::{
    collections::{HashMap, HashSet},
    fs,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result, bail};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::{
    EVIDENCE_SCHEMA_VERSION, SOURCE_REVISION,
    protocol::{
        ActionId, DeliveryDisposition, ErrorClass, IdentityVerdict, PredicateVerdict,
        sample_binding,
    },
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum InputSource {
    DriverCurrentAttempt,
    User,
    Unknown,
    ObserverLost,
    None,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum DispatchBoundary {
    NotDispatched,
    FrameFlushed,
    ResponseReceived,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum CleanupVerdict {
    NotRequired,
    Released,
    Residual,
    Unproven,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ControlState {
    Running,
    Preempted,
    CancelledBeforeDispatch,
    Quarantined,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ActionResult {
    Observed,
    Rejected,
    Unknown,
    NotDispatched,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Outcome {
    Pass,
    Fail,
    EnvironmentUnavailable,
    Unsupported,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceScope {
    ContractOnly,
    Native,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct CleanupProbeEvidence {
    pub process_tree_gone: bool,
    pub desktop_handles_gone: bool,
    pub input_hooks_gone: bool,
    pub permissions_released: bool,
    pub temp_state_removed: bool,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct DistributionEvidence {
    pub dependency_manifest_sha256: String,
    pub sbom_sha256: String,
    pub license_manifest_sha256: String,
    pub artifact_provenance_sha256: String,
    pub production_identity_verified: bool,
    pub signature_verified: bool,
    pub notarization_verified: bool,
    pub clean_machine_runtime_verified: bool,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Evidence {
    pub schema_version: String,
    pub evidence_scope: EvidenceScope,
    pub source_revision: String,
    pub build_manifest_sha256: String,
    pub bridge_contract_sha256: String,
    pub artifact_sha256: String,
    pub platform: String,
    pub arch: String,
    pub sample_id: String,
    pub action_id: ActionId,
    pub predicate_id: String,
    pub predicate_sha256: String,
    pub oracle_id: String,
    pub oracle_sha256: String,
    pub attempt_id: String,
    pub worker_instance_id: String,
    pub fence: u64,
    pub generation_before: u64,
    pub generation_after: u64,
    pub input_source: InputSource,
    pub dispatch_boundary: DispatchBoundary,
    pub delivery_disposition: DeliveryDisposition,
    pub error_class: ErrorClass,
    pub observe_identity: IdentityVerdict,
    pub observe_predicate: PredicateVerdict,
    pub stable_observation_count: u8,
    pub stable_observation_interval_ms: u64,
    pub host_oracle_independent: bool,
    pub cleanup_verdict: CleanupVerdict,
    pub cleanup_probes: Option<CleanupProbeEvidence>,
    pub distribution: Option<DistributionEvidence>,
    pub control_state: ControlState,
    pub action_result: ActionResult,
    pub global_quarantine: bool,
    pub elapsed_ms: u64,
    pub retry_count: u8,
    pub outcome: Outcome,
}

#[derive(Deserialize)]
struct Allowlist {
    schema_version: String,
    actions: Vec<AllowedAction>,
    forbidden: Vec<String>,
}
#[derive(Deserialize)]
struct AllowedAction {
    id: ActionId,
    allowed_samples: Vec<String>,
    requires_qualified_ref: bool,
}

pub struct VerificationInputs {
    pub allowlist: PathBuf,
    pub definitions: PathBuf,
    pub build_manifest: PathBuf,
    pub bridge_contract: PathBuf,
    pub artifact: PathBuf,
    pub contract_fixture_artifact: PathBuf,
    pub contract_fixture_provenance: PathBuf,
    pub distribution: Option<DistributionInputs>,
}

pub struct DistributionInputs {
    pub dependency_manifest: PathBuf,
    pub sbom: PathBuf,
    pub license_manifest: PathBuf,
    pub artifact_provenance: PathBuf,
}

pub fn sha256(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
pub fn file_sha256(path: &Path) -> Result<String> {
    Ok(sha256(&fs::read(path)?))
}

pub fn definition_sha256(definitions: &Path, group: &str, id: &str) -> Result<String> {
    let value: serde_json::Value = serde_json::from_slice(&fs::read(definitions)?)?;
    let definition = value[group]
        .as_array()
        .context("定义组不存在")?
        .iter()
        .find(|entry| entry["id"] == id)
        .context("冻结定义不存在")?;
    Ok(sha256(&serde_json::to_vec(definition)?))
}

pub fn action_allowed(allowlist_json: &[u8], sample_id: &str, action_id: ActionId) -> Result<bool> {
    let allowlist: Allowlist = serde_json::from_slice(allowlist_json)?;
    Ok(allowlist.actions.iter().any(|action| {
        action.id == action_id
            && action
                .allowed_samples
                .iter()
                .any(|sample| sample == sample_id)
    }))
}

pub fn verify_file(path: &Path, inputs: &VerificationInputs) -> Result<()> {
    let evidence: Evidence = serde_json::from_slice(
        &fs::read(path).with_context(|| format!("读取 {}", path.display()))?,
    )?;
    let allowlist: Allowlist = serde_json::from_slice(&fs::read(&inputs.allowlist)?)?;
    verify(&evidence, &allowlist, inputs)
}

fn verify(e: &Evidence, allowlist: &Allowlist, inputs: &VerificationInputs) -> Result<()> {
    if e.schema_version != EVIDENCE_SCHEMA_VERSION
        || e.source_revision != SOURCE_REVISION
        || e.retry_count != 0
    {
        bail!("版本、revision 或 retry_count 不合约");
    }
    if !["windows", "macos"].contains(&e.platform.as_str())
        || !["x86_64", "aarch64"].contains(&e.arch.as_str())
    {
        bail!("平台或架构不受支持");
    }
    if e.evidence_scope == EvidenceScope::Native {
        if file_sha256(&inputs.artifact)? == file_sha256(&inputs.contract_fixture_artifact)? {
            bail!("native evidence 不得复用 contract fixture artifact");
        }
        if let Some(distribution) = &inputs.distribution
            && file_sha256(&distribution.artifact_provenance)?
                == file_sha256(&inputs.contract_fixture_provenance)?
        {
            bail!("native evidence 不得复用 contract fixture provenance");
        }
    }
    let binding = sample_binding(&e.sample_id).context("未知 sample")?;
    if binding.action_id != e.action_id
        || binding.predicate_id != e.predicate_id
        || binding.oracle_id != e.oracle_id
    {
        bail!("sample/action/predicate/oracle 错配");
    }
    let expected_hashes = [
        file_sha256(&inputs.build_manifest)?,
        file_sha256(&inputs.bridge_contract)?,
        file_sha256(&inputs.artifact)?,
        definition_sha256(&inputs.definitions, "predicates", &e.predicate_id)?,
        definition_sha256(&inputs.definitions, "oracles", &e.oracle_id)?,
    ];
    let actual_hashes = [
        &e.build_manifest_sha256,
        &e.bridge_contract_sha256,
        &e.artifact_sha256,
        &e.predicate_sha256,
        &e.oracle_sha256,
    ];
    if actual_hashes
        .into_iter()
        .zip(expected_hashes.iter())
        .any(|(actual, expected)| actual != expected)
    {
        bail!("证据哈希未绑定实际输入");
    }
    if allowlist.schema_version != "agent-desktop-spike-action-allowlist/v1"
        || allowlist.forbidden.is_empty()
    {
        bail!("allowlist 版本或拒绝集无效");
    }
    let action = allowlist
        .actions
        .iter()
        .find(|action| action.id == e.action_id)
        .context("action 不在 allowlist")?;
    if !action
        .allowed_samples
        .iter()
        .any(|sample| sample == &e.sample_id)
        || action.requires_qualified_ref && e.action_id == ActionId::ActionNone
    {
        bail!("sample/action allowlist 错配");
    }
    if e.action_result == ActionResult::Observed
        && !(e.delivery_disposition == DeliveryDisposition::DeliveredVerified
            && e.error_class == ErrorClass::None
            && e.observe_identity == IdentityVerdict::Match
            && e.observe_predicate == PredicateVerdict::Satisfied
            && e.dispatch_boundary == DispatchBoundary::ResponseReceived
            && e.generation_before == e.generation_after
            && matches!(
                e.input_source,
                InputSource::None | InputSource::DriverCurrentAttempt
            )
            && e.stable_observation_count >= 2
            && (250..=500).contains(&e.stable_observation_interval_ms)
            && e.host_oracle_independent)
    {
        bail!("observed 缺少完整验证");
    }
    if e.action_result == ActionResult::NotDispatched
        && !(e.dispatch_boundary == DispatchBoundary::NotDispatched
            && e.delivery_disposition == DeliveryDisposition::NotDelivered)
    {
        bail!("not_dispatched 边界不一致");
    }
    if matches!(
        e.cleanup_verdict,
        CleanupVerdict::Residual | CleanupVerdict::Unproven
    ) && !(e.global_quarantine
        && e.control_state == ControlState::Quarantined
        && e.outcome == Outcome::Fail)
    {
        bail!("残留未 quarantine");
    }
    if e.global_quarantine
        && !matches!(
            e.cleanup_verdict,
            CleanupVerdict::Residual | CleanupVerdict::Unproven
        )
    {
        bail!("quarantine 缺少残留裁决");
    }
    if e.cleanup_verdict == CleanupVerdict::Released
        && !e.cleanup_probes.as_ref().is_some_and(|p| {
            p.process_tree_gone
                && p.desktop_handles_gone
                && p.input_hooks_gone
                && p.permissions_released
                && p.temp_state_removed
        })
    {
        bail!("Released 缺少完整 cleanup probes");
    }
    if e.outcome == Outcome::Pass {
        if e.sample_id == "DIST-01" {
            verify_distribution_files(
                e.distribution.as_ref().context("DIST pass 缺少分发证据")?,
                inputs
                    .distribution
                    .as_ref()
                    .context("DIST pass 缺少 verifier 分发输入")?,
            )?;
        }
        verify_pass_profile(e)?;
    }
    Ok(())
}

fn verify_distribution_files(e: &DistributionEvidence, inputs: &DistributionInputs) -> Result<()> {
    let expected = [
        file_sha256(&inputs.dependency_manifest)?,
        file_sha256(&inputs.sbom)?,
        file_sha256(&inputs.license_manifest)?,
        file_sha256(&inputs.artifact_provenance)?,
    ];
    let actual = [
        &e.dependency_manifest_sha256,
        &e.sbom_sha256,
        &e.license_manifest_sha256,
        &e.artifact_provenance_sha256,
    ];
    if actual.into_iter().zip(expected.iter()).any(|(a, b)| a != b) {
        bail!("DIST 责任链哈希未绑定 verifier 实际输入文件");
    }
    Ok(())
}

fn verify_pass_profile(e: &Evidence) -> Result<()> {
    match e.sample_id.as_str() {
        "IDENTITY-01" | "ELEMENT-01" | "APPSET-01" | "APPSET-02" | "FOCUS-01" | "DISPLAY-01" => {
            if e.action_result != ActionResult::Observed
                || e.error_class != ErrorClass::None
                || e.control_state != ControlState::Running
            {
                bail!("正常动作 pass profile 错误");
            }
        }
        "IDENTITY-02" => {
            if e.action_result != ActionResult::Rejected
                || e.delivery_disposition != DeliveryDisposition::NotDelivered
                || !matches!(
                    e.error_class,
                    ErrorClass::Identity | ErrorClass::Stale | ErrorClass::Ambiguous
                )
                || !matches!(
                    e.observe_identity,
                    IdentityVerdict::Mismatch | IdentityVerdict::Stale | IdentityVerdict::Ambiguous
                )
            {
                bail!("身份拒绝 pass profile 错误");
            }
        }
        "PERMISSION-01" => {
            if !(e.action_result == ActionResult::Rejected
                && e.delivery_disposition == DeliveryDisposition::NotDelivered
                && e.error_class == ErrorClass::Permission)
            {
                bail!("权限拒绝 pass profile 错误");
            }
        }
        "INPUT-01" => {
            if !(e.input_source == InputSource::User
                && e.control_state == ControlState::Preempted
                && e.action_result == ActionResult::NotDispatched)
            {
                bail!("用户抢占 pass profile 错误");
            }
        }
        "INPUT-02" => {
            if !(e.input_source == InputSource::DriverCurrentAttempt
                && e.control_state == ControlState::Running
                && e.action_result == ActionResult::Observed)
            {
                bail!("合成输入 pass profile 错误");
            }
        }
        "INPUT-03" => {
            if !matches!(
                e.input_source,
                InputSource::Unknown | InputSource::ObserverLost
            ) || e.control_state != ControlState::Preempted
                || e.action_result != ActionResult::NotDispatched
            {
                bail!("observer fail-closed profile 错误");
            }
        }
        "CANCEL-01" => {
            if !(e.control_state == ControlState::CancelledBeforeDispatch
                && e.action_result == ActionResult::NotDispatched)
            {
                bail!("flush 前取消 profile 错误");
            }
        }
        "CANCEL-02" => {
            if !(e.action_result == ActionResult::Unknown
                && matches!(
                    e.delivery_disposition,
                    DeliveryDisposition::DeliveredUnverified | DeliveryDisposition::Unknown
                )
                && e.error_class == ErrorClass::Driver
                && e.cleanup_verdict == CleanupVerdict::Released)
            {
                bail!("在途终止 profile 错误");
            }
        }
        "FAULT-01" => {
            if !(e.action_result == ActionResult::Unknown
                && matches!(
                    e.delivery_disposition,
                    DeliveryDisposition::DeliveredUnverified | DeliveryDisposition::Unknown
                )
                && matches!(e.error_class, ErrorClass::Deadline | ErrorClass::Driver)
                && e.cleanup_verdict == CleanupVerdict::Released)
            {
                bail!("故障 profile 错误");
            }
        }
        "DIST-01" => {
            if !(e.action_id == ActionId::ActionNone
                && e.action_result == ActionResult::NotDispatched
                && e.cleanup_verdict == CleanupVerdict::NotRequired
                && e.error_class == ErrorClass::None
                && e.distribution.as_ref().is_some_and(|d| {
                    d.production_identity_verified
                        && d.signature_verified
                        && d.notarization_verified
                        && d.clean_machine_runtime_verified
                        && [
                            &d.dependency_manifest_sha256,
                            &d.sbom_sha256,
                            &d.license_manifest_sha256,
                            &d.artifact_provenance_sha256,
                        ]
                        .into_iter()
                        .all(|v| v.len() == 64)
                }))
            {
                bail!("分发 profile 错误");
            }
        }
        _ => bail!("未知 sample"),
    }
    if !matches!(
        e.cleanup_verdict,
        CleanupVerdict::NotRequired | CleanupVerdict::Released
    ) || e.global_quarantine
    {
        bail!("pass 不得带残留");
    }
    Ok(())
}

pub fn verify_catalog_inputs(
    allowlist_path: &Path,
    definitions_path: &Path,
    catalog_path: &Path,
) -> Result<()> {
    let allowlist: Allowlist = serde_json::from_slice(&fs::read(allowlist_path)?)?;
    let definitions: serde_json::Value = serde_json::from_slice(&fs::read(definitions_path)?)?;
    let predicates: HashSet<&str> = definitions["predicates"]
        .as_array()
        .context("predicates")?
        .iter()
        .filter_map(|value| value["id"].as_str())
        .collect();
    let oracles: HashSet<&str> = definitions["oracles"]
        .as_array()
        .context("oracles")?
        .iter()
        .filter_map(|value| value["id"].as_str())
        .collect();
    let action_map: HashMap<&str, ActionId> = allowlist
        .actions
        .iter()
        .flat_map(|action| {
            action
                .allowed_samples
                .iter()
                .map(move |sample| (sample.as_str(), action.id))
        })
        .collect();
    let catalog = fs::read_to_string(catalog_path)?;
    let mut rows = 0;
    for line in catalog
        .lines()
        .filter(|line| line.starts_with("| `") && !line.contains("Sample ID"))
    {
        let cells: Vec<_> = line.split('|').map(str::trim).collect();
        if cells.len() < 8 {
            bail!("样本目录行格式无效");
        }
        let sample = cells[1].trim_matches('`');
        let action: ActionId =
            serde_json::from_str(&format!("\"{}\"", cells[4].trim_matches('`')))?;
        let predicate = cells[5].trim_matches('`');
        let oracle = cells[6].trim_matches('`');
        let binding = sample_binding(sample).context("协议缺少样本绑定")?;
        if action_map.get(sample) != Some(&action)
            || action != binding.action_id
            || predicate != binding.predicate_id
            || oracle != binding.oracle_id
            || !predicates.contains(predicate)
            || !oracles.contains(oracle)
        {
            bail!("样本 {sample} 的 action/predicate/oracle 错配");
        }
        rows += 1;
    }
    if rows != 15 {
        bail!("预期 15 个冻结样本，实际 {rows}");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    fn root() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../openspec/changes/cu-s4-agent-desktop-driver-spike")
    }
    fn inputs() -> VerificationInputs {
        let root = root();
        VerificationInputs {
            allowlist: root.join("action-allowlist-v1.json"),
            definitions: root.join("sample-definitions-v1.json"),
            build_manifest: Path::new(env!("CARGO_MANIFEST_DIR")).join("build-manifest.json"),
            bridge_contract: root.join("bridge-contract-v1.md"),
            artifact: root.join("schema-fixtures/fixture-artifact.txt"),
            contract_fixture_artifact: root.join("schema-fixtures/fixture-artifact.txt"),
            contract_fixture_provenance: root.join("schema-fixtures/fixture-provenance.json"),
            distribution: None,
        }
    }
    #[test]
    fn frozen_catalog_is_internally_consistent() {
        let root = root();
        assert!(
            verify_catalog_inputs(
                &root.join("action-allowlist-v1.json"),
                &root.join("sample-definitions-v1.json"),
                &root.join("sample-catalog.md")
            )
            .is_ok()
        );
    }
    #[test]
    fn only_valid_fixture_passes_with_actual_hashes() {
        let root = root();
        assert!(verify_file(&root.join("schema-fixtures/observed-pass.json"), &inputs()).is_ok());
        for name in [
            "invalid-element-action-pass.json",
            "invalid-element-unknown-pass.json",
            "invalid-observe-pass.json",
            "invalid-error-class-pass.json",
            "invalid-hash-pass.json",
        ] {
            assert!(
                verify_file(&root.join("schema-fixtures").join(name), &inputs()).is_err(),
                "{name}"
            );
        }
    }
    #[test]
    fn dist_hashes_are_bound_to_actual_verifier_files() {
        let Ok(dir) = tempfile::tempdir() else {
            panic!("tempdir")
        };
        let paths = DistributionInputs {
            dependency_manifest: dir.path().join("dependencies.json"),
            sbom: dir.path().join("sbom.json"),
            license_manifest: dir.path().join("licenses.json"),
            artifact_provenance: dir.path().join("provenance.json"),
        };
        for (path, bytes) in [
            (&paths.dependency_manifest, b"dependencies".as_slice()),
            (&paths.sbom, b"sbom".as_slice()),
            (&paths.license_manifest, b"licenses".as_slice()),
            (&paths.artifact_provenance, b"provenance".as_slice()),
        ] {
            assert!(fs::write(path, bytes).is_ok());
        }
        let evidence = DistributionEvidence {
            dependency_manifest_sha256: file_sha256(&paths.dependency_manifest).unwrap_or_default(),
            sbom_sha256: file_sha256(&paths.sbom).unwrap_or_default(),
            license_manifest_sha256: file_sha256(&paths.license_manifest).unwrap_or_default(),
            artifact_provenance_sha256: file_sha256(&paths.artifact_provenance).unwrap_or_default(),
            production_identity_verified: true,
            signature_verified: true,
            notarization_verified: true,
            clean_machine_runtime_verified: true,
        };
        assert!(verify_distribution_files(&evidence, &paths).is_ok());
        assert!(fs::write(&paths.sbom, b"changed").is_ok());
        assert!(verify_distribution_files(&evidence, &paths).is_err());
    }

    #[test]
    fn native_scope_cannot_reuse_contract_fixture_artifact() {
        let root = root();
        let Ok(bytes) = fs::read(root.join("schema-fixtures/observed-pass.json")) else {
            panic!("fixture")
        };
        let Ok(mut evidence) = serde_json::from_slice::<Evidence>(&bytes) else {
            panic!("fixture parse")
        };
        evidence.evidence_scope = EvidenceScope::Native;
        let Ok(allowlist) = serde_json::from_slice::<Allowlist>(
            &fs::read(root.join("action-allowlist-v1.json")).unwrap_or_default(),
        ) else {
            panic!("allowlist")
        };
        assert!(verify(&evidence, &allowlist, &inputs()).is_err());
    }
}
