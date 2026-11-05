use crate::protocol::{
    DeliveryDisposition, ErrorClass, HostRequest, IdentityVerdict, PredicateVerdict, Status,
};

#[derive(Debug)]
pub struct DispatchReply {
    pub status: Status,
    pub error_class: ErrorClass,
    pub disposition: DeliveryDisposition,
    pub driver_status: &'static str,
}

#[derive(Debug)]
pub struct SnapshotReply {
    pub status: Status,
    pub error_class: ErrorClass,
    pub snapshot_id: String,
    pub qualified_refs: Vec<String>,
    pub identity: IdentityVerdict,
}

#[derive(Debug)]
pub struct ObserveReply {
    pub status: Status,
    pub error_class: ErrorClass,
    pub snapshot_id: String,
    pub identity: IdentityVerdict,
    pub predicate: PredicateVerdict,
    pub element_count: u16,
}

pub trait Backend {
    fn snapshot(&mut self, request: &HostRequest) -> SnapshotReply;
    fn dispatch(&mut self, request: &HostRequest) -> DispatchReply;
    fn observe(&mut self, request: &HostRequest) -> ObserveReply;
}

pub struct MockBackend;

impl Backend for MockBackend {
    fn snapshot(&mut self, _request: &HostRequest) -> SnapshotReply {
        SnapshotReply {
            status: Status::Ok,
            error_class: ErrorClass::None,
            snapshot_id: "snap".into(),
            qualified_refs: vec!["@snap:e1".into()],
            identity: IdentityVerdict::Match,
        }
    }
    fn dispatch(&mut self, _request: &HostRequest) -> DispatchReply {
        DispatchReply {
            status: Status::Ok,
            error_class: ErrorClass::None,
            disposition: DeliveryDisposition::DeliveredVerified,
            driver_status: "mock_contract_only_delivery_requires_host_observe",
        }
    }
    fn observe(&mut self, _request: &HostRequest) -> ObserveReply {
        ObserveReply {
            status: Status::Ok,
            error_class: ErrorClass::None,
            snapshot_id: "mock-snapshot".into(),
            identity: IdentityVerdict::Match,
            predicate: PredicateVerdict::Satisfied,
            element_count: 1,
        }
    }
}

#[cfg(any(target_os = "windows", target_os = "macos"))]
pub mod native {
    //! 固定 revision 的公开 core/adapter API 接线。真实动作只在已授权的原生样本运行中启用。
    use super::*;

    pub struct NativeBackend {
        #[cfg(target_os = "windows")]
        adapter: agent_desktop_windows::WindowsAdapter,
        #[cfg(target_os = "macos")]
        adapter: agent_desktop_macos::MacOSAdapter,
    }

    impl NativeBackend {
        pub fn new() -> Result<Self, agent_desktop_core::AdapterError> {
            #[cfg(target_os = "windows")]
            {
                agent_desktop_windows::ensure_owned_process_mta_and_dpi()?;
                agent_desktop_core::install_private_file_ops(Box::new(
                    agent_desktop_windows::WindowsPrivateFile,
                ))
                .map_err(|_| {
                    agent_desktop_core::AdapterError::internal(
                        "Windows private file backend already installed",
                    )
                })?;
            }
            #[cfg(target_os = "macos")]
            agent_desktop_macos::ensure_cocoa_multithreaded()?;
            Ok(Self {
                #[cfg(target_os = "windows")]
                adapter: agent_desktop_windows::WindowsAdapter::new(),
                #[cfg(target_os = "macos")]
                adapter: agent_desktop_macos::MacOSAdapter::new(),
            })
        }
    }

    impl Backend for NativeBackend {
        fn snapshot(&mut self, _request: &HostRequest) -> SnapshotReply {
            let _adapter: &dyn agent_desktop_core::PlatformAdapter = &self.adapter;
            SnapshotReply {
                status: Status::Rejected,
                error_class: ErrorClass::Unsupported,
                snapshot_id: String::new(),
                qualified_refs: Vec::new(),
                identity: IdentityVerdict::Unavailable,
            }
        }
        fn dispatch(&mut self, _request: &HostRequest) -> DispatchReply {
            // 上游 public command API 要求持久 RefStore/CommandContext。它只能由冻结样本的
            // 私有 state root 建立；在原生 runner 完成前拒绝副作用，不能退回 CLI/C ABI。
            let _adapter: &dyn agent_desktop_core::PlatformAdapter = &self.adapter;
            DispatchReply {
                status: Status::Rejected,
                error_class: ErrorClass::Unsupported,
                disposition: DeliveryDisposition::NotDelivered,
                driver_status: "native_target_identity_gate_not_proven",
            }
        }
        fn observe(&mut self, _request: &HostRequest) -> ObserveReply {
            let _adapter: &dyn agent_desktop_core::PlatformAdapter = &self.adapter;
            ObserveReply {
                status: Status::Rejected,
                error_class: ErrorClass::Unsupported,
                snapshot_id: String::new(),
                identity: IdentityVerdict::Unavailable,
                predicate: PredicateVerdict::Unavailable,
                element_count: 0,
            }
        }
    }
}
