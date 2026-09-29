//! 执行未知结果的单一事实：内部枚举与 wire 枚举的唯一定点映射。
//! wire 形状（kebab-case 序列化）由 `yonder-protocol` 派生；SQLite 持久化
//! 字符串由 `yonder-adapters` 依同一枚举维护，三方不再各写一份转换。

/// 内部有界分类；语义与 [`yonder_protocol::AttemptUnknownReason`] 一一对应。
#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum UnknownReason {
    InvalidInput,
    DependencyUnavailable,
    WorkerFailed,
    TimedOut,
    InvalidResponse,
    IdentityMismatch,
    ObserveFailed,
    UserInput,
}

/// 唯一的内部 → wire 映射；新增 reason 时必须同步本实现，漏映射即编译失败。
impl From<UnknownReason> for yonder_protocol::AttemptUnknownReason {
    fn from(reason: UnknownReason) -> Self {
        match reason {
            UnknownReason::InvalidInput => Self::InvalidInput,
            UnknownReason::DependencyUnavailable => Self::DependencyUnavailable,
            UnknownReason::WorkerFailed => Self::WorkerFailed,
            UnknownReason::TimedOut => Self::TimedOut,
            UnknownReason::InvalidResponse => Self::InvalidResponse,
            UnknownReason::IdentityMismatch => Self::IdentityMismatch,
            UnknownReason::ObserveFailed => Self::ObserveFailed,
            UnknownReason::UserInput => Self::UserInput,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 锁定内部枚举到 wire 枚举的逐值映射；序列化形状由协议派生测试保护。
    #[test]
    fn internal_reasons_map_one_to_one_to_wire_reasons() {
        let pairs = [
            (UnknownReason::InvalidInput, "invalid-input"),
            (UnknownReason::DependencyUnavailable, "dependency-unavailable"),
            (UnknownReason::WorkerFailed, "worker-failed"),
            (UnknownReason::TimedOut, "timed-out"),
            (UnknownReason::InvalidResponse, "invalid-response"),
            (UnknownReason::IdentityMismatch, "identity-mismatch"),
            (UnknownReason::ObserveFailed, "observe-failed"),
            (UnknownReason::UserInput, "user-input"),
        ];
        for (internal, wire_name) in pairs {
            let wire: yonder_protocol::AttemptUnknownReason = internal.into();
            assert_eq!(
                serde_json::to_value(wire).unwrap(),
                serde_json::Value::String(wire_name.into()),
                "{wire_name} 映射漂移"
            );
        }
    }
}
