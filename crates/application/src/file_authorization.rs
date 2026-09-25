use crate::{
    AuthContext, Status, Task,
    file::{
        FileCreateTargetRequest, FileError, FileIdentity, FilePort, FileReadRequest,
        inspect_create_target, read,
    },
};
use std::{collections::BTreeMap, sync::Mutex};

pub const MAX_FILE_GRANTS: usize = 256;
pub const MAX_FILE_GRANT_LIFETIME_MS: u64 = 15 * 60 * 1000;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FileGrantPurpose {
    Read,
    CreateNew,
    Replace,
    Trash,
}

impl FileGrantPurpose {
    fn one_shot(self) -> bool {
        !matches!(self, Self::Read)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum FileGrantLocation {
    Existing {
        request: FileReadRequest,
        canonical_path: String,
        identity: FileIdentity,
        sha256: String,
    },
    CreateTarget {
        request: FileCreateTargetRequest,
        canonical_path: String,
        parent_identity: FileIdentity,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FileGrant {
    pub grant_id: String,
    pub task_id: String,
    pub owner_agent_id: String,
    pub purpose: FileGrantPurpose,
    pub expires_at_ms: u64,
    pub location: FileGrantLocation,
}

/// 可跨 Gateway/UI 传递的授权摘要；不包含任何文件位置或内容事实。
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FileGrantSummary {
    pub grant_id: String,
    pub purpose: FileGrantPurpose,
    pub expires_at_ms: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FileGrantError {
    InvalidInput,
    PermissionDenied,
    NotFound,
    Expired,
    Capacity,
    Unavailable,
    File(FileError),
}

impl From<FileError> for FileGrantError {
    fn from(error: FileError) -> Self {
        Self::File(error)
    }
}

#[derive(Default)]
pub struct FileAuthorizationRegistry {
    grants: Mutex<BTreeMap<String, FileGrant>>,
}

impl FileAuthorizationRegistry {
    pub fn issue_existing(
        &self,
        files: &dyn FilePort,
        auth: AuthContext<'_>,
        task: &Task,
        grant_id: &str,
        purpose: FileGrantPurpose,
        request: &FileReadRequest,
        now_ms: u64,
        expires_at_ms: u64,
    ) -> Result<FileGrant, FileGrantError> {
        validate_issue(auth, task, grant_id, now_ms, expires_at_ms)?;
        if !matches!(
            purpose,
            FileGrantPurpose::Read | FileGrantPurpose::Replace | FileGrantPurpose::Trash
        ) {
            return Err(FileGrantError::InvalidInput);
        }
        let snapshot = read(files, request)?;
        let canonical_path = snapshot.canonical_path;
        self.register(
            FileGrant {
                grant_id: grant_id.into(),
                task_id: task.id.clone(),
                owner_agent_id: task.owner_agent_id.clone(),
                purpose,
                expires_at_ms,
                location: FileGrantLocation::Existing {
                    request: FileReadRequest {
                        path: canonical_path.clone(),
                        authorized_root: request.authorized_root.clone(),
                    },
                    canonical_path,
                    identity: snapshot.identity,
                    sha256: snapshot.sha256,
                },
            },
            now_ms,
        )
    }

    pub fn issue_create_target(
        &self,
        files: &dyn FilePort,
        auth: AuthContext<'_>,
        task: &Task,
        grant_id: &str,
        request: &FileCreateTargetRequest,
        now_ms: u64,
        expires_at_ms: u64,
    ) -> Result<FileGrant, FileGrantError> {
        validate_issue(auth, task, grant_id, now_ms, expires_at_ms)?;
        let target = inspect_create_target(files, request)?;
        let canonical_path = target.canonical_path;
        self.register(
            FileGrant {
                grant_id: grant_id.into(),
                task_id: task.id.clone(),
                owner_agent_id: task.owner_agent_id.clone(),
                purpose: FileGrantPurpose::CreateNew,
                expires_at_ms,
                location: FileGrantLocation::CreateTarget {
                    request: FileCreateTargetRequest {
                        path: canonical_path.clone(),
                        authorized_root: request.authorized_root.clone(),
                    },
                    canonical_path,
                    parent_identity: target.parent_identity,
                },
            },
            now_ms,
        )
    }

    pub fn resolve(
        &self,
        auth: AuthContext<'_>,
        task: &Task,
        grant_id: &str,
        purpose: FileGrantPurpose,
        now_ms: u64,
    ) -> Result<FileGrant, FileGrantError> {
        if !matches!(auth, AuthContext::Agent(_))
            || !crate::valid_id(auth.agent_id())
            || !crate::valid_id(grant_id)
            || !active_task(task)
        {
            return Err(FileGrantError::PermissionDenied);
        }
        let mut grants = self
            .grants
            .lock()
            .map_err(|_| FileGrantError::Unavailable)?;
        let grant = grants
            .get(grant_id)
            .cloned()
            .ok_or(FileGrantError::NotFound)?;
        if grant.expires_at_ms <= now_ms {
            grants.remove(grant_id);
            return Err(FileGrantError::Expired);
        }
        if auth.agent_id() != task.owner_agent_id
            || grant.owner_agent_id != task.owner_agent_id
            || grant.task_id != task.id
            || grant.purpose != purpose
        {
            return Err(FileGrantError::PermissionDenied);
        }
        if purpose.one_shot() {
            grants.remove(grant_id);
        }
        Ok(grant)
    }

    /// 已认证归属 Agent 的只读视图；不会解析或消费一次性授权。
    pub fn list_for_task(
        &self,
        auth: AuthContext<'_>,
        task: &Task,
        now_ms: u64,
    ) -> Result<Vec<FileGrantSummary>, FileGrantError> {
        if !matches!(auth, AuthContext::Agent(_))
            || !crate::valid_id(auth.agent_id())
            || !active_task(task)
            || auth.agent_id() != task.owner_agent_id
        {
            return Err(FileGrantError::PermissionDenied);
        }
        let mut grants = self
            .grants
            .lock()
            .map_err(|_| FileGrantError::Unavailable)?;
        grants.retain(|_, grant| grant.expires_at_ms > now_ms);
        Ok(grants
            .values()
            .filter(|grant| {
                grant.task_id == task.id && grant.owner_agent_id == task.owner_agent_id
            })
            .map(|grant| FileGrantSummary {
                grant_id: grant.grant_id.clone(),
                purpose: grant.purpose,
                expires_at_ms: grant.expires_at_ms,
            })
            .collect())
    }

    /// 可信本机界面的当前任务视图；同样不包含位置或内容事实。
    pub fn list_for_local(
        &self,
        auth: AuthContext<'_>,
        task: &Task,
        now_ms: u64,
    ) -> Result<Vec<FileGrantSummary>, FileGrantError> {
        require_local(auth)?;
        if !active_task(task) {
            return Err(FileGrantError::PermissionDenied);
        }
        let mut grants = self
            .grants
            .lock()
            .map_err(|_| FileGrantError::Unavailable)?;
        grants.retain(|_, grant| grant.expires_at_ms > now_ms);
        Ok(grants
            .values()
            .filter(|grant| {
                grant.task_id == task.id && grant.owner_agent_id == task.owner_agent_id
            })
            .map(|grant| FileGrantSummary {
                grant_id: grant.grant_id.clone(),
                purpose: grant.purpose,
                expires_at_ms: grant.expires_at_ms,
            })
            .collect())
    }

    pub fn revoke(
        &self,
        auth: AuthContext<'_>,
        task: &Task,
        grant_id: &str,
    ) -> Result<(), FileGrantError> {
        require_local(auth)?;
        let mut grants = self
            .grants
            .lock()
            .map_err(|_| FileGrantError::Unavailable)?;
        let grant = grants.get(grant_id).ok_or(FileGrantError::NotFound)?;
        if grant.task_id != task.id || grant.owner_agent_id != task.owner_agent_id {
            return Err(FileGrantError::PermissionDenied);
        }
        grants.remove(grant_id);
        Ok(())
    }

    pub fn revoke_owner(
        &self,
        auth: AuthContext<'_>,
        owner_agent_id: &str,
    ) -> Result<usize, FileGrantError> {
        require_local(auth)?;
        if !crate::valid_id(owner_agent_id) {
            return Err(FileGrantError::InvalidInput);
        }
        let mut grants = self
            .grants
            .lock()
            .map_err(|_| FileGrantError::Unavailable)?;
        let before = grants.len();
        grants.retain(|_, grant| grant.owner_agent_id != owner_agent_id);
        Ok(before - grants.len())
    }

    fn register(&self, grant: FileGrant, now_ms: u64) -> Result<FileGrant, FileGrantError> {
        let mut grants = self
            .grants
            .lock()
            .map_err(|_| FileGrantError::Unavailable)?;
        grants.retain(|_, existing| existing.expires_at_ms > now_ms);
        if grants.contains_key(&grant.grant_id) {
            return Err(FileGrantError::InvalidInput);
        }
        if grants.len() >= MAX_FILE_GRANTS {
            return Err(FileGrantError::Capacity);
        }
        grants.insert(grant.grant_id.clone(), grant.clone());
        Ok(grant)
    }
}

fn validate_issue(
    auth: AuthContext<'_>,
    task: &Task,
    grant_id: &str,
    now_ms: u64,
    expires_at_ms: u64,
) -> Result<(), FileGrantError> {
    require_local(auth)?;
    if !crate::valid_id(&task.id)
        || !crate::valid_id(&task.owner_agent_id)
        || !crate::valid_id(grant_id)
        || !active_task(task)
        || expires_at_ms <= now_ms
        || expires_at_ms
            .checked_sub(now_ms)
            .is_none_or(|duration| duration > MAX_FILE_GRANT_LIFETIME_MS)
    {
        return Err(FileGrantError::InvalidInput);
    }
    Ok(())
}

fn require_local(auth: AuthContext<'_>) -> Result<(), FileGrantError> {
    if matches!(auth, AuthContext::LocalUser(_)) {
        Ok(())
    } else {
        Err(FileGrantError::PermissionDenied)
    }
}

fn active_task(task: &Task) -> bool {
    !matches!(
        task.status,
        Status::Completed | Status::Failed | Status::Cancelled
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{TaskSource, file::*};
    use std::sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    };

    struct Port(AtomicUsize);

    impl FilePort for Port {
        fn read(&self, request: &FileReadRequest) -> Result<FileSnapshot, FileError> {
            self.0.fetch_add(1, Ordering::Relaxed);
            Ok(FileSnapshot {
                canonical_path: request.path.clone(),
                identity: FileIdentity {
                    volume_id: 1,
                    file_id: 2,
                },
                sha256: "1".repeat(64),
                bytes: b"must not be retained".to_vec(),
            })
        }

        fn inspect_create_target(
            &self,
            request: &FileCreateTargetRequest,
        ) -> Result<FileCreateTarget, FileError> {
            self.0.fetch_add(1, Ordering::Relaxed);
            Ok(FileCreateTarget {
                canonical_path: request.path.clone(),
                parent_identity: FileIdentity {
                    volume_id: 1,
                    file_id: 3,
                },
            })
        }

        fn write_atomic(
            &self,
            _: &FileWriteRequest,
            _: &dyn FileValidator,
        ) -> Result<FileWriteReceipt, FileError> {
            panic!("授权核心不得写文件")
        }

        fn write_atomic_guarded(
            &self,
            _: &FileSourceGuard,
            _: &FileWriteRequest,
            _: &dyn FileValidator,
        ) -> Result<FileWriteReceipt, FileError> {
            panic!("授权核心不得写文件")
        }

        fn trash(
            &self,
            _: &FileTrashRequest,
            _: &LocalTrashAuthorization,
        ) -> Result<FileTrashReceipt, FileError> {
            panic!("授权核心不得移动文件")
        }
    }

    fn task(id: &str, owner: &str) -> Task {
        Task {
            id: id.into(),
            owner_agent_id: owner.into(),
            name: None,
            source: TaskSource::LocalAgent,
            status: Status::Created,
            sequence: 1,
        }
    }

    fn existing() -> FileReadRequest {
        FileReadRequest {
            path: "/tmp/root/source.docx".into(),
            authorized_root: "/tmp/root".into(),
        }
    }

    #[test]
    fn only_local_user_issues_and_records_no_file_bytes() {
        let port = Port(AtomicUsize::new(0));
        let registry = FileAuthorizationRegistry::default();
        let task = task("task-a", "agent-a");
        assert_eq!(
            registry.issue_existing(
                &port,
                AuthContext::Agent("agent-a"),
                &task,
                "grant-a",
                FileGrantPurpose::Read,
                &existing(),
                100,
                200,
            ),
            Err(FileGrantError::PermissionDenied)
        );
        assert_eq!(port.0.load(Ordering::Relaxed), 0);
        assert_eq!(
            registry.issue_existing(
                &port,
                AuthContext::LocalUser("desktop"),
                &task,
                "grant-too-long",
                FileGrantPurpose::Read,
                &existing(),
                100,
                100 + MAX_FILE_GRANT_LIFETIME_MS + 1,
            ),
            Err(FileGrantError::InvalidInput)
        );
        assert_eq!(port.0.load(Ordering::Relaxed), 0);
        let grant = registry
            .issue_existing(
                &port,
                AuthContext::LocalUser("desktop"),
                &task,
                "grant-a",
                FileGrantPurpose::Read,
                &existing(),
                100,
                200,
            )
            .unwrap();
        assert!(matches!(
            grant.location,
            FileGrantLocation::Existing { ref sha256, .. } if sha256 == &"1".repeat(64)
        ));
        assert_eq!(port.0.load(Ordering::Relaxed), 1);
    }

    #[test]
    fn resolution_is_task_agent_purpose_and_expiry_bound() {
        let port = Port(AtomicUsize::new(0));
        let registry = FileAuthorizationRegistry::default();
        let first = task("task-a", "agent-a");
        registry
            .issue_existing(
                &port,
                AuthContext::LocalUser("desktop"),
                &first,
                "grant-read",
                FileGrantPurpose::Read,
                &existing(),
                100,
                200,
            )
            .unwrap();
        assert_eq!(
            registry.resolve(
                AuthContext::Agent("agent-b"),
                &first,
                "grant-read",
                FileGrantPurpose::Read,
                150,
            ),
            Err(FileGrantError::PermissionDenied)
        );
        assert_eq!(
            registry.resolve(
                AuthContext::Agent("agent-a"),
                &task("task-b", "agent-a"),
                "grant-read",
                FileGrantPurpose::Read,
                150,
            ),
            Err(FileGrantError::PermissionDenied)
        );
        assert_eq!(
            registry.resolve(
                AuthContext::Agent("agent-a"),
                &first,
                "grant-read",
                FileGrantPurpose::Replace,
                150,
            ),
            Err(FileGrantError::PermissionDenied)
        );
        assert!(
            registry
                .resolve(
                    AuthContext::Agent("agent-a"),
                    &first,
                    "grant-read",
                    FileGrantPurpose::Read,
                    150,
                )
                .is_ok()
        );
        assert!(
            registry
                .resolve(
                    AuthContext::Agent("agent-a"),
                    &first,
                    "grant-read",
                    FileGrantPurpose::Read,
                    199,
                )
                .is_ok()
        );
        assert_eq!(
            registry.resolve(
                AuthContext::Agent("agent-a"),
                &first,
                "grant-read",
                FileGrantPurpose::Read,
                200,
            ),
            Err(FileGrantError::Expired)
        );
    }

    #[test]
    fn safe_lists_expose_no_location_and_do_not_consume_write_grants() {
        let port = Port(AtomicUsize::new(0));
        let registry = FileAuthorizationRegistry::default();
        let task = task("task-a", "agent-a");
        registry
            .issue_create_target(
                &port,
                AuthContext::LocalUser("desktop"),
                &task,
                "grant-create",
                &FileCreateTargetRequest {
                    path: "/tmp/root/output.docx".into(),
                    authorized_root: "/tmp/root".into(),
                },
                100,
                200,
            )
            .unwrap();
        let summaries = registry
            .list_for_task(AuthContext::Agent("agent-a"), &task, 150)
            .unwrap();
        assert_eq!(
            summaries,
            vec![FileGrantSummary {
                grant_id: "grant-create".into(),
                purpose: FileGrantPurpose::CreateNew,
                expires_at_ms: 200,
            }]
        );
        assert!(registry
            .resolve(
                AuthContext::Agent("agent-a"),
                &task,
                "grant-create",
                FileGrantPurpose::CreateNew,
                150,
            )
            .is_ok());
        assert_eq!(
            registry.list_for_task(AuthContext::Agent("agent-b"), &task, 150),
            Err(FileGrantError::PermissionDenied)
        );
    }

    #[test]
    fn write_grant_is_consumed_once_even_under_concurrency() {
        let port = Port(AtomicUsize::new(0));
        let registry = Arc::new(FileAuthorizationRegistry::default());
        let task = task("task-a", "agent-a");
        registry
            .issue_create_target(
                &port,
                AuthContext::LocalUser("desktop"),
                &task,
                "grant-create",
                &FileCreateTargetRequest {
                    path: "/tmp/root/output.docx".into(),
                    authorized_root: "/tmp/root".into(),
                },
                100,
                200,
            )
            .unwrap();
        let successes = (0..8)
            .map(|_| {
                let registry = Arc::clone(&registry);
                let task = task.clone();
                std::thread::spawn(move || {
                    registry
                        .resolve(
                            AuthContext::Agent("agent-a"),
                            &task,
                            "grant-create",
                            FileGrantPurpose::CreateNew,
                            150,
                        )
                        .is_ok()
                })
            })
            .map(|thread| usize::from(thread.join().unwrap()))
            .sum::<usize>();
        assert_eq!(successes, 1);
        assert_eq!(
            registry.resolve(
                AuthContext::Agent("agent-a"),
                &task,
                "grant-create",
                FileGrantPurpose::CreateNew,
                150,
            ),
            Err(FileGrantError::NotFound)
        );
    }

    #[test]
    fn revocation_and_capacity_fail_closed() {
        let port = Port(AtomicUsize::new(0));
        let registry = FileAuthorizationRegistry::default();
        let first = task("task-a", "agent-a");
        registry
            .issue_existing(
                &port,
                AuthContext::LocalUser("desktop"),
                &first,
                "grant-revoke",
                FileGrantPurpose::Read,
                &existing(),
                100,
                200,
            )
            .unwrap();
        assert_eq!(
            registry.revoke(AuthContext::LocalUser("desktop"), &first, "grant-revoke",),
            Ok(())
        );
        assert_eq!(
            registry.resolve(
                AuthContext::Agent("agent-a"),
                &first,
                "grant-revoke",
                FileGrantPurpose::Read,
                150,
            ),
            Err(FileGrantError::NotFound)
        );
        registry
            .issue_existing(
                &port,
                AuthContext::LocalUser("desktop"),
                &first,
                "grant-owner-revoke",
                FileGrantPurpose::Read,
                &existing(),
                100,
                200,
            )
            .unwrap();
        assert_eq!(
            registry.revoke_owner(AuthContext::LocalUser("desktop"), "agent-a"),
            Ok(1)
        );
        assert_eq!(
            registry.resolve(
                AuthContext::Agent("agent-a"),
                &first,
                "grant-owner-revoke",
                FileGrantPurpose::Read,
                150,
            ),
            Err(FileGrantError::NotFound)
        );

        for index in 0..MAX_FILE_GRANTS {
            registry
                .issue_existing(
                    &port,
                    AuthContext::LocalUser("desktop"),
                    &first,
                    &format!("grant-{index}"),
                    FileGrantPurpose::Read,
                    &existing(),
                    100,
                    200,
                )
                .unwrap();
        }
        assert_eq!(
            registry.issue_existing(
                &port,
                AuthContext::LocalUser("desktop"),
                &first,
                "grant-overflow",
                FileGrantPurpose::Read,
                &existing(),
                100,
                200,
            ),
            Err(FileGrantError::Capacity)
        );
    }
}
