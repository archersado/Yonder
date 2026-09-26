use serde::Deserialize;

#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
struct ReleaseContract {
    version: String,
    protocol: ProtocolVersion,
    sqlite_schema: i64,
}

#[derive(Deserialize)]
struct ProtocolVersion {
    major: u16,
    minor: u16,
}

pub fn validate() -> Result<(), String> {
    validate_contract(include_str!("../release-contract.json"))
}

fn validate_contract(text: &str) -> Result<(), String> {
    let contract: ReleaseContract = serde_json::from_str(text).map_err(|_| "发布契约不可用")?;
    if contract.version != env!("CARGO_PKG_VERSION") {
        return Err("桌面版本与发布契约不一致".into());
    }
    let protocol = yonder_application::gateway::PROTOCOL_VERSION;
    if contract.protocol.major != protocol.major || contract.protocol.minor != protocol.minor {
        return Err("协议版本与发布契约不一致".into());
    }
    if contract.sqlite_schema != yonder_adapters::task_store::SQLITE_SCHEMA_VERSION {
        return Err("SQLite schema 与发布契约不一致".into());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn contract_matches_current_runtime_versions() {
        assert!(validate().is_ok());
    }

    #[test]
    fn mismatched_contract_is_rejected() {
        let contract = r#"{
            "version": "0.1.0",
            "protocol": {"major": 1, "minor": 29},
            "sqlite_schema": 13
        }"#;
        assert_eq!(validate_contract(contract).unwrap_err(), "SQLite schema 与发布契约不一致");
    }

    #[test]
    fn mismatched_version_is_rejected() {
        let contract = r#"{
            "version": "0.0.0",
            "protocol": {"major": 1, "minor": 29},
            "sqlite_schema": 19
        }"#;
        assert_eq!(validate_contract(contract).unwrap_err(), "桌面版本与发布契约不一致");
    }

    #[test]
    fn mismatched_protocol_is_rejected() {
        let contract = r#"{
            "version": "0.1.0",
            "protocol": {"major": 1, "minor": 999},
            "sqlite_schema": 19
        }"#;
        assert_eq!(validate_contract(contract).unwrap_err(), "协议版本与发布契约不一致");
    }
}
