use serde::{Deserialize, Serialize};

pub const JEV_REMOTE_ENDPOINT: &str = "https://api.typesafe.ai";

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum JevServiceMode { Local, Remote }

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum JevCapability { Cua, Bua, Document, Command }

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct JevConfig {
    pub enabled: bool,
    pub service_mode: JevServiceMode,
    pub endpoint: String,
    pub step_limit: u32,
    pub time_limit_ms: u64,
    pub token_limit: u64,
    pub capabilities: Vec<JevCapability>,
}

#[derive(Debug, Eq, PartialEq)]
pub enum JevConfigError {
    InvalidEndpoint,
    InvalidStepLimit,
    InvalidTimeLimit,
    InvalidTokenLimit,
    InvalidCapability,
}

impl Default for JevConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            service_mode: JevServiceMode::Local,
            endpoint: "http://localhost".into(),
            step_limit: 10,
            time_limit_ms: 60_000,
            token_limit: 10_000,
            capabilities: vec![JevCapability::Cua, JevCapability::Bua, JevCapability::Document, JevCapability::Command],
        }
    }
}

impl JevConfig {
    pub fn validate(&self) -> Result<(), JevConfigError> {
        let (scheme, rest) = self.endpoint.split_once("://").ok_or(JevConfigError::InvalidEndpoint)?;
        let authority = rest.split('/').next().unwrap_or_default();
        let contains_credentials = authority.contains('@');
        let contains_fragment = rest.contains('#');
        let contains_whitespace = self.endpoint.chars().any(char::is_whitespace);
        let remote_requires_https = self.service_mode == JevServiceMode::Remote && scheme != "https";
        let unsupported_remote = self.service_mode == JevServiceMode::Remote
            && !matches!(self.endpoint.as_str(), JEV_REMOTE_ENDPOINT | "https://api.typesafe.ai/");
        if !matches!(scheme, "http" | "https")
            || authority.is_empty()
            || contains_credentials
            || contains_fragment
            || contains_whitespace
            || rest.contains('?')
            || remote_requires_https
            || unsupported_remote
        {
            return Err(JevConfigError::InvalidEndpoint);
        }
        if !(1..=100).contains(&self.step_limit) { return Err(JevConfigError::InvalidStepLimit); }
        if !(1_000..=600_000).contains(&self.time_limit_ms) { return Err(JevConfigError::InvalidTimeLimit); }
        if !(1..=100_000).contains(&self.token_limit) { return Err(JevConfigError::InvalidTokenLimit); }
        let mut capabilities = self.capabilities.clone();
        capabilities.sort_unstable();
        capabilities.dedup();
        if capabilities.len() != self.capabilities.len() || capabilities.is_empty() {
            return Err(JevConfigError::InvalidCapability);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_default_config() {
        assert_eq!(JevConfig::default().validate(), Ok(()));
    }

    #[test]
    fn rejects_invalid_config() {
        let mut config = JevConfig { capabilities: vec![JevCapability::Cua], ..JevConfig::default() };
        config.endpoint = "localhost".into();
        assert_eq!(config.validate(), Err(JevConfigError::InvalidEndpoint));
        config.endpoint = "http://localhost".into();
        config.step_limit = 0;
        assert_eq!(config.validate(), Err(JevConfigError::InvalidStepLimit));
        config.step_limit = 10;
        config.time_limit_ms = 999;
        assert_eq!(config.validate(), Err(JevConfigError::InvalidTimeLimit));
        config.time_limit_ms = 60_000;
        config.token_limit = 0;
        assert_eq!(config.validate(), Err(JevConfigError::InvalidTokenLimit));
        config.token_limit = 10_000;
        config.capabilities = Vec::new();
        assert_eq!(config.validate(), Err(JevConfigError::InvalidCapability));
    }

    #[test]
    fn rejects_endpoint_query_strings() {
        let config = JevConfig {
            endpoint: "http://localhost/jev?token=secret".into(),
            ..JevConfig::default()
        };
        assert_eq!(config.validate(), Err(JevConfigError::InvalidEndpoint));
    }

    #[test]
    fn rejects_unapproved_remote_endpoint() {
        let config = JevConfig {
            service_mode: JevServiceMode::Remote,
            endpoint: "https://example.com".into(),
            ..JevConfig::default()
        };
        assert_eq!(config.validate(), Err(JevConfigError::InvalidEndpoint));
    }

    #[test]
    fn rejects_endpoint_credentials_and_remote_http() {
        let local_with_credentials = JevConfig {
            endpoint: "http://user:pass@localhost".into(),
            ..JevConfig::default()
        };
        assert_eq!(local_with_credentials.validate(), Err(JevConfigError::InvalidEndpoint));

        let remote_http = JevConfig {
            service_mode: JevServiceMode::Remote,
            endpoint: "http://localhost".into(),
            ..JevConfig::default()
        };
        assert_eq!(remote_http.validate(), Err(JevConfigError::InvalidEndpoint));

        let remote_https = JevConfig {
            service_mode: JevServiceMode::Remote,
            endpoint: JEV_REMOTE_ENDPOINT.into(),
            ..JevConfig::default()
        };
        assert_eq!(remote_https.validate(), Ok(()));
    }
}
