use crate::AuthContext;

pub const MAX_FILE_PATH_BYTES: usize = 4096;
pub const MAX_FILE_CONTENT_BYTES: usize = 16 * 1024 * 1024;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct FileIdentity {
    pub volume_id: u64,
    pub file_id: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FileReadRequest {
    pub path: String,
    pub authorized_root: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FileSnapshot {
    pub canonical_path: String,
    pub identity: FileIdentity,
    pub sha256: String,
    pub bytes: Vec<u8>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FileCreateTargetRequest {
    pub path: String,
    pub authorized_root: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FileCreateTarget {
    pub canonical_path: String,
    pub parent_identity: FileIdentity,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum FileWriteMode {
    CreateNew {
        expected_parent_identity: FileIdentity,
    },
    Replace {
        expected_identity: FileIdentity,
        expected_sha256: String,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FileWriteRequest {
    pub path: String,
    pub authorized_root: String,
    pub bytes: Vec<u8>,
    pub mode: FileWriteMode,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FileWriteReceipt {
    pub canonical_path: String,
    pub identity: FileIdentity,
    pub sha256: String,
    pub bytes_written: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FileSourceGuard {
    pub source: FileReadRequest,
    pub expected_identity: FileIdentity,
    pub expected_sha256: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FileTrashRequest {
    pub path: String,
    pub authorized_root: String,
    pub expected_identity: FileIdentity,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FileTrashReceipt {
    pub identity: FileIdentity,
}

pub struct LocalTrashAuthorization {
    _private: (),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FileError {
    InvalidInput,
    OutsideAuthorizedRoot,
    NotFound,
    AlreadyExists,
    TooLarge,
    IdentityChanged,
    ContentChanged,
    Busy,
    HostLocked,
    ValidationFailed,
    PermissionDenied,
    UnsupportedPlatform,
    IoFailed,
    Unknown,
}

pub trait FileValidator: Send + Sync {
    fn validate(&self, staged_bytes: &[u8]) -> bool;
}

pub struct AcceptAnyFile;

impl FileValidator for AcceptAnyFile {
    fn validate(&self, _: &[u8]) -> bool {
        true
    }
}

pub trait FilePort {
    fn read(&self, request: &FileReadRequest) -> Result<FileSnapshot, FileError>;

    fn inspect_create_target(
        &self,
        request: &FileCreateTargetRequest,
    ) -> Result<FileCreateTarget, FileError>;

    fn write_atomic(
        &self,
        request: &FileWriteRequest,
        validator: &dyn FileValidator,
    ) -> Result<FileWriteReceipt, FileError>;

    fn write_atomic_guarded(
        &self,
        source: &FileSourceGuard,
        request: &FileWriteRequest,
        validator: &dyn FileValidator,
    ) -> Result<FileWriteReceipt, FileError>;

    fn trash(
        &self,
        request: &FileTrashRequest,
        authorization: &LocalTrashAuthorization,
    ) -> Result<FileTrashReceipt, FileError>;
}

fn valid_path(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= MAX_FILE_PATH_BYTES
        && !value.contains('\0')
        && std::path::Path::new(value).is_absolute()
}

fn valid_hash(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn validate_location(path: &str, authorized_root: &str) -> Result<(), FileError> {
    if !valid_path(path) || !valid_path(authorized_root) || path == authorized_root {
        return Err(FileError::InvalidInput);
    }
    Ok(())
}

pub fn read(port: &dyn FilePort, request: &FileReadRequest) -> Result<FileSnapshot, FileError> {
    validate_location(&request.path, &request.authorized_root)?;
    port.read(request)
}

pub fn inspect_create_target(
    port: &dyn FilePort,
    request: &FileCreateTargetRequest,
) -> Result<FileCreateTarget, FileError> {
    validate_location(&request.path, &request.authorized_root)?;
    port.inspect_create_target(request)
}

pub fn write_atomic(
    port: &dyn FilePort,
    request: &FileWriteRequest,
    validator: &dyn FileValidator,
) -> Result<FileWriteReceipt, FileError> {
    validate_write(request)?;
    port.write_atomic(request, validator)
}

pub fn write_atomic_guarded(
    port: &dyn FilePort,
    source: &FileSourceGuard,
    request: &FileWriteRequest,
    validator: &dyn FileValidator,
) -> Result<FileWriteReceipt, FileError> {
    validate_location(&source.source.path, &source.source.authorized_root)?;
    if !valid_hash(&source.expected_sha256) {
        return Err(FileError::InvalidInput);
    }
    validate_write(request)?;
    port.write_atomic_guarded(source, request, validator)
}

fn validate_write(request: &FileWriteRequest) -> Result<(), FileError> {
    validate_location(&request.path, &request.authorized_root)?;
    if request.bytes.len() > MAX_FILE_CONTENT_BYTES {
        return Err(FileError::TooLarge);
    }
    if matches!(
        &request.mode,
        FileWriteMode::Replace {
            expected_sha256,
            ..
        } if !valid_hash(expected_sha256)
    ) {
        return Err(FileError::InvalidInput);
    }
    Ok(())
}

pub fn trash(
    port: &dyn FilePort,
    auth: AuthContext<'_>,
    request: &FileTrashRequest,
) -> Result<FileTrashReceipt, FileError> {
    if !matches!(auth, AuthContext::LocalUser(_)) {
        return Err(FileError::PermissionDenied);
    }
    validate_location(&request.path, &request.authorized_root)?;
    port.trash(request, &LocalTrashAuthorization { _private: () })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

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
                sha256: "0".repeat(64),
                bytes: Vec::new(),
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
                    file_id: 4,
                },
            })
        }

        fn write_atomic(
            &self,
            request: &FileWriteRequest,
            _: &dyn FileValidator,
        ) -> Result<FileWriteReceipt, FileError> {
            self.0.fetch_add(1, Ordering::Relaxed);
            Ok(FileWriteReceipt {
                canonical_path: request.path.clone(),
                identity: FileIdentity {
                    volume_id: 1,
                    file_id: 3,
                },
                sha256: "0".repeat(64),
                bytes_written: request.bytes.len() as u64,
            })
        }

        fn write_atomic_guarded(
            &self,
            _: &FileSourceGuard,
            request: &FileWriteRequest,
            validator: &dyn FileValidator,
        ) -> Result<FileWriteReceipt, FileError> {
            self.write_atomic(request, validator)
        }

        fn trash(
            &self,
            request: &FileTrashRequest,
            _: &LocalTrashAuthorization,
        ) -> Result<FileTrashReceipt, FileError> {
            self.0.fetch_add(1, Ordering::Relaxed);
            Ok(FileTrashReceipt {
                identity: request.expected_identity,
            })
        }
    }

    fn location() -> FileReadRequest {
        FileReadRequest {
            path: "/tmp/root/file".into(),
            authorized_root: "/tmp/root".into(),
        }
    }

    #[test]
    fn application_rejects_invalid_input_and_agent_trash_before_port() {
        let port = Port(AtomicUsize::new(0));
        let mut request = location();
        request.path = "relative".into();
        assert_eq!(read(&port, &request), Err(FileError::InvalidInput));

        let write = FileWriteRequest {
            path: "/tmp/root/file".into(),
            authorized_root: "/tmp/root".into(),
            bytes: Vec::new(),
            mode: FileWriteMode::Replace {
                expected_identity: FileIdentity {
                    volume_id: 1,
                    file_id: 2,
                },
                expected_sha256: "not-a-hash".into(),
            },
        };
        assert_eq!(
            write_atomic(&port, &write, &AcceptAnyFile),
            Err(FileError::InvalidInput)
        );

        let trash_request = FileTrashRequest {
            path: "/tmp/root/file".into(),
            authorized_root: "/tmp/root".into(),
            expected_identity: FileIdentity {
                volume_id: 1,
                file_id: 2,
            },
        };
        assert_eq!(
            trash(&port, AuthContext::Agent("agent-a"), &trash_request),
            Err(FileError::PermissionDenied)
        );
        assert_eq!(port.0.load(Ordering::Relaxed), 0);
        assert!(read(&port, &location()).is_ok());
        assert!(trash(&port, AuthContext::LocalUser("desktop"), &trash_request).is_ok());
        assert_eq!(port.0.load(Ordering::Relaxed), 2);
    }
}
