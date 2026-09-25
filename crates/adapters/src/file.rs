use yonder_application::file::{
    FileCreateTarget, FileCreateTargetRequest, FileError, FilePort, FileReadRequest, FileSnapshot,
    FileSourceGuard, FileTrashReceipt, FileTrashRequest, FileValidator, FileWriteReceipt,
    FileWriteRequest, LocalTrashAuthorization,
};

pub struct ControlledFileAdapter {
    #[cfg(target_os = "macos")]
    leases: std::sync::Mutex<std::collections::HashSet<macos::LeaseKey>>,
    #[cfg(target_os = "macos")]
    next_temporary: std::sync::atomic::AtomicU64,
    #[cfg(all(test, target_os = "macos"))]
    fail_parent_sync: bool,
}

impl Default for ControlledFileAdapter {
    fn default() -> Self {
        Self {
            #[cfg(target_os = "macos")]
            leases: std::sync::Mutex::new(std::collections::HashSet::new()),
            #[cfg(target_os = "macos")]
            next_temporary: std::sync::atomic::AtomicU64::new(0),
            #[cfg(all(test, target_os = "macos"))]
            fail_parent_sync: false,
        }
    }
}

#[cfg(not(target_os = "macos"))]
impl FilePort for ControlledFileAdapter {
    fn read(&self, _: &FileReadRequest) -> Result<FileSnapshot, FileError> {
        Err(FileError::UnsupportedPlatform)
    }

    fn inspect_create_target(
        &self,
        _: &FileCreateTargetRequest,
    ) -> Result<FileCreateTarget, FileError> {
        Err(FileError::UnsupportedPlatform)
    }

    fn write_atomic(
        &self,
        _: &FileWriteRequest,
        _: &dyn FileValidator,
    ) -> Result<FileWriteReceipt, FileError> {
        Err(FileError::UnsupportedPlatform)
    }

    fn write_atomic_guarded(
        &self,
        _: &FileSourceGuard,
        _: &FileWriteRequest,
        _: &dyn FileValidator,
    ) -> Result<FileWriteReceipt, FileError> {
        Err(FileError::UnsupportedPlatform)
    }

    fn trash(
        &self,
        _: &FileTrashRequest,
        _: &LocalTrashAuthorization,
    ) -> Result<FileTrashReceipt, FileError> {
        Err(FileError::UnsupportedPlatform)
    }
}

#[cfg(target_os = "macos")]
mod macos {
    use super::*;
    use fs2::FileExt;
    use objc2_foundation::{NSFileManager, NSString, NSURL};
    use sha2::{Digest, Sha256};
    use std::{
        collections::HashSet,
        ffi::CString,
        fs::{self, File, OpenOptions},
        io::{Read, Seek, SeekFrom, Write},
        os::unix::{
            ffi::OsStrExt,
            fs::{MetadataExt, OpenOptionsExt},
        },
        path::{Path, PathBuf},
        sync::{Mutex, MutexGuard, atomic::Ordering},
    };
    use yonder_application::file::{
        FileIdentity, FileWriteMode, MAX_FILE_CONTENT_BYTES, MAX_FILE_PATH_BYTES,
    };

    const RENAME_EXCL: u32 = 0x0000_0004;
    const RENAME_NOFOLLOW_ANY: u32 = 0x0000_0010;

    unsafe extern "C" {
        fn renamex_np(
            from: *const std::ffi::c_char,
            to: *const std::ffi::c_char,
            flags: u32,
        ) -> i32;
    }

    #[derive(Clone, Debug, Eq, Hash, PartialEq)]
    pub(super) enum LeaseKey {
        Existing(FileIdentity),
        New { parent: FileIdentity, name: Vec<u8> },
    }

    pub(super) struct Lease<'a> {
        leases: &'a Mutex<HashSet<LeaseKey>>,
        key: Option<LeaseKey>,
    }

    impl Drop for Lease<'_> {
        fn drop(&mut self) {
            if let (Ok(mut leases), Some(key)) = (self.leases.lock(), self.key.take()) {
                leases.remove(&key);
            }
        }
    }

    struct Temporary {
        path: PathBuf,
        committed: bool,
    }

    impl Drop for Temporary {
        fn drop(&mut self) {
            if !self.committed {
                let _ = fs::remove_file(&self.path);
            }
        }
    }

    fn io_error(error: std::io::Error) -> FileError {
        match error.kind() {
            std::io::ErrorKind::NotFound => FileError::NotFound,
            std::io::ErrorKind::AlreadyExists => FileError::AlreadyExists,
            std::io::ErrorKind::PermissionDenied => FileError::PermissionDenied,
            _ => FileError::IoFailed,
        }
    }

    fn valid_path(value: &str) -> bool {
        !value.is_empty()
            && value.len() <= MAX_FILE_PATH_BYTES
            && !value.contains('\0')
            && Path::new(value).is_absolute()
    }

    fn validate_location(path: &str, root: &str) -> Result<(), FileError> {
        if !valid_path(path) || !valid_path(root) || path == root {
            return Err(FileError::InvalidInput);
        }
        Ok(())
    }

    fn valid_hash(value: &str) -> bool {
        value.len() == 64
            && value
                .bytes()
                .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    }

    fn identity(metadata: &fs::Metadata) -> FileIdentity {
        FileIdentity {
            volume_id: metadata.dev(),
            file_id: metadata.ino(),
        }
    }

    fn canonical_root(value: &str) -> Result<PathBuf, FileError> {
        let root = fs::canonicalize(value).map_err(io_error)?;
        if !root.is_dir() {
            return Err(FileError::InvalidInput);
        }
        Ok(root)
    }

    fn inside(root: &Path, path: &Path) -> Result<(), FileError> {
        if !path.starts_with(root) {
            return Err(FileError::OutsideAuthorizedRoot);
        }
        Ok(())
    }

    fn existing(root: &Path, value: &str) -> Result<(PathBuf, fs::Metadata), FileError> {
        let path = fs::canonicalize(value).map_err(io_error)?;
        inside(root, &path)?;
        let metadata = fs::metadata(&path).map_err(io_error)?;
        if !metadata.is_file() {
            return Err(FileError::InvalidInput);
        }
        Ok((path, metadata))
    }

    fn new_target(root: &Path, value: &str) -> Result<(PathBuf, FileIdentity, Vec<u8>), FileError> {
        let requested = Path::new(value);
        let parent = requested.parent().ok_or(FileError::InvalidInput)?;
        let parent = fs::canonicalize(parent).map_err(io_error)?;
        inside(root, &parent)?;
        if !parent.is_dir() {
            return Err(FileError::InvalidInput);
        }
        let name = requested
            .file_name()
            .filter(|name| !name.is_empty())
            .ok_or(FileError::InvalidInput)?;
        let target = parent.join(name);
        if fs::symlink_metadata(&target).is_ok() {
            return Err(FileError::AlreadyExists);
        }
        let parent_identity = identity(&fs::metadata(&parent).map_err(io_error)?);
        Ok((target, parent_identity, name.as_bytes().to_vec()))
    }

    pub(super) fn acquire<'a>(
        leases: &'a Mutex<HashSet<LeaseKey>>,
        key: LeaseKey,
    ) -> Result<Lease<'a>, FileError> {
        let mut values: MutexGuard<'_, HashSet<LeaseKey>> =
            leases.lock().map_err(|_| FileError::Unknown)?;
        if !values.insert(key.clone()) {
            return Err(FileError::Busy);
        }
        drop(values);
        Ok(Lease {
            leases,
            key: Some(key),
        })
    }

    fn hash(bytes: &[u8]) -> String {
        let digest = Sha256::digest(bytes);
        let mut value = String::with_capacity(64);
        for byte in digest {
            use std::fmt::Write as _;
            let _ = write!(value, "{byte:02x}");
        }
        value
    }

    fn bounded_read(file: &mut File) -> Result<Vec<u8>, FileError> {
        file.seek(SeekFrom::Start(0)).map_err(io_error)?;
        let mut bytes = Vec::new();
        file.take((MAX_FILE_CONTENT_BYTES + 1) as u64)
            .read_to_end(&mut bytes)
            .map_err(io_error)?;
        if bytes.len() > MAX_FILE_CONTENT_BYTES {
            return Err(FileError::TooLarge);
        }
        Ok(bytes)
    }

    fn snapshot(path: &Path) -> Result<FileSnapshot, FileError> {
        let mut file = OpenOptions::new().read(true).open(path).map_err(io_error)?;
        FileExt::try_lock_shared(&file).map_err(|_| FileError::HostLocked)?;
        let before = file.metadata().map_err(io_error)?;
        if before.len() > MAX_FILE_CONTENT_BYTES as u64 {
            return Err(FileError::TooLarge);
        }
        let bytes = bounded_read(&mut file)?;
        let after = file.metadata().map_err(io_error)?;
        if identity(&before) != identity(&after)
            || before.len() != after.len()
            || after.len() != bytes.len() as u64
        {
            return Err(FileError::IdentityChanged);
        }
        FileExt::unlock(&file).map_err(io_error)?;
        let canonical_path = path.to_str().ok_or(FileError::InvalidInput)?.to_owned();
        Ok(FileSnapshot {
            canonical_path,
            identity: identity(&after),
            sha256: hash(&bytes),
            bytes,
        })
    }

    fn create_temporary(
        adapter: &ControlledFileAdapter,
        parent: &Path,
    ) -> Result<(Temporary, File), FileError> {
        for _ in 0..32 {
            let suffix = adapter.next_temporary.fetch_add(1, Ordering::Relaxed) + 1;
            let path = parent.join(format!(".yonder-{}-{suffix}.tmp", std::process::id()));
            match OpenOptions::new()
                .read(true)
                .write(true)
                .create_new(true)
                .mode(0o600)
                .open(&path)
            {
                Ok(file) => {
                    return Ok((
                        Temporary {
                            path,
                            committed: false,
                        },
                        file,
                    ));
                }
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
                Err(error) => return Err(io_error(error)),
            }
        }
        Err(FileError::Busy)
    }

    fn rename_exclusive(from: &Path, to: &Path) -> Result<(), FileError> {
        let from =
            CString::new(from.as_os_str().as_bytes()).map_err(|_| FileError::InvalidInput)?;
        let to = CString::new(to.as_os_str().as_bytes()).map_err(|_| FileError::InvalidInput)?;
        // SAFETY: both C strings remain live for the call and contain no interior NUL.
        let result = unsafe {
            renamex_np(
                from.as_ptr(),
                to.as_ptr(),
                RENAME_EXCL | RENAME_NOFOLLOW_ANY,
            )
        };
        if result == 0 {
            Ok(())
        } else {
            Err(io_error(std::io::Error::last_os_error()))
        }
    }

    pub(super) fn trash_impl(
        adapter: &ControlledFileAdapter,
        request: &FileTrashRequest,
    ) -> Result<(FileTrashReceipt, PathBuf), FileError> {
        validate_location(&request.path, &request.authorized_root)?;
        let root = canonical_root(&request.authorized_root)?;
        let (path, metadata) = existing(&root, &request.path)?;
        let file_identity = identity(&metadata);
        if file_identity != request.expected_identity {
            return Err(FileError::IdentityChanged);
        }
        let _lease = acquire(&adapter.leases, LeaseKey::Existing(file_identity))?;
        let file = OpenOptions::new()
            .read(true)
            .open(&path)
            .map_err(io_error)?;
        FileExt::try_lock_exclusive(&file).map_err(|_| FileError::HostLocked)?;
        if identity(&file.metadata().map_err(io_error)?) != file_identity {
            return Err(FileError::IdentityChanged);
        }
        let path_string = path.to_str().ok_or(FileError::InvalidInput)?;
        let url = NSURL::fileURLWithPath(&NSString::from_str(path_string));
        let mut resulting_url = None;
        NSFileManager::defaultManager()
            .trashItemAtURL_resultingItemURL_error(&url, Some(&mut resulting_url))
            .map_err(|_| FileError::IoFailed)?;
        let resulting_path = resulting_url
            .and_then(|url| url.path())
            .map(|path| PathBuf::from(path.to_string()))
            .ok_or(FileError::Unknown)?;
        let resulting_identity = fs::metadata(&resulting_path)
            .map(|metadata| identity(&metadata))
            .map_err(|_| FileError::Unknown)?;
        if path.exists() || resulting_identity != file_identity {
            return Err(FileError::Unknown);
        }
        Ok((
            FileTrashReceipt {
                identity: file_identity,
            },
            resulting_path,
        ))
    }

    fn ensure_current(
        path: &Path,
        expected_identity: FileIdentity,
        expected_hash: &str,
    ) -> Result<File, FileError> {
        let mut file = OpenOptions::new()
            .read(true)
            .write(true)
            .open(path)
            .map_err(io_error)?;
        FileExt::try_lock_exclusive(&file).map_err(|_| FileError::HostLocked)?;
        let metadata = file.metadata().map_err(io_error)?;
        if identity(&metadata) != expected_identity {
            return Err(FileError::IdentityChanged);
        }
        let bytes = bounded_read(&mut file)?;
        if hash(&bytes) != expected_hash {
            return Err(FileError::ContentChanged);
        }
        Ok(file)
    }

    fn verify_source(
        file: &mut File,
        requested_path: &str,
        root: &Path,
        expected_identity: FileIdentity,
        expected_hash: &str,
    ) -> Result<(), FileError> {
        let (current_path, current_metadata) = existing(root, requested_path)?;
        if identity(&current_metadata) != expected_identity {
            return Err(FileError::IdentityChanged);
        }
        let descriptor_metadata = file.metadata().map_err(io_error)?;
        if identity(&descriptor_metadata) != expected_identity {
            return Err(FileError::IdentityChanged);
        }
        let descriptor_path = fs::canonicalize(requested_path).map_err(io_error)?;
        if descriptor_path != current_path {
            return Err(FileError::IdentityChanged);
        }
        let bytes = bounded_read(file)?;
        if hash(&bytes) != expected_hash {
            return Err(FileError::ContentChanged);
        }
        Ok(())
    }

    struct GuardedValidator<'a> {
        file: Mutex<File>,
        requested_path: String,
        root: PathBuf,
        expected_identity: FileIdentity,
        expected_hash: String,
        inner: &'a dyn FileValidator,
        error: Mutex<Option<FileError>>,
    }

    impl GuardedValidator<'_> {
        fn verify(&self) -> Result<(), FileError> {
            let mut file = self.file.lock().map_err(|_| FileError::Unknown)?;
            verify_source(
                &mut file,
                &self.requested_path,
                &self.root,
                self.expected_identity,
                &self.expected_hash,
            )
        }

        fn reject(&self, error: FileError) -> bool {
            if let Ok(mut stored) = self.error.lock() {
                *stored = Some(error);
            }
            false
        }

        fn take_error(&self) -> Option<FileError> {
            self.error.lock().ok().and_then(|mut error| error.take())
        }
    }

    impl FileValidator for GuardedValidator<'_> {
        fn validate(&self, staged_bytes: &[u8]) -> bool {
            if let Err(error) = self.verify() {
                return self.reject(error);
            }
            if !self.inner.validate(staged_bytes) {
                return false;
            }
            match self.verify() {
                Ok(()) => true,
                Err(error) => self.reject(error),
            }
        }
    }

    impl FilePort for ControlledFileAdapter {
        fn read(&self, request: &FileReadRequest) -> Result<FileSnapshot, FileError> {
            validate_location(&request.path, &request.authorized_root)?;
            let root = canonical_root(&request.authorized_root)?;
            let (path, metadata) = existing(&root, &request.path)?;
            if metadata.len() > MAX_FILE_CONTENT_BYTES as u64 {
                return Err(FileError::TooLarge);
            }
            snapshot(&path)
        }

        fn inspect_create_target(
            &self,
            request: &FileCreateTargetRequest,
        ) -> Result<FileCreateTarget, FileError> {
            validate_location(&request.path, &request.authorized_root)?;
            let root = canonical_root(&request.authorized_root)?;
            let (target, parent_identity, _) = new_target(&root, &request.path)?;
            Ok(FileCreateTarget {
                canonical_path: target.to_str().ok_or(FileError::InvalidInput)?.to_owned(),
                parent_identity,
            })
        }

        fn write_atomic(
            &self,
            request: &FileWriteRequest,
            validator: &dyn FileValidator,
        ) -> Result<FileWriteReceipt, FileError> {
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
            let root = canonical_root(&request.authorized_root)?;
            let (target, lease_key) = match &request.mode {
                FileWriteMode::CreateNew {
                    expected_parent_identity,
                } => {
                    let (target, parent, name) = new_target(&root, &request.path)?;
                    if parent != *expected_parent_identity {
                        return Err(FileError::IdentityChanged);
                    }
                    (target, LeaseKey::New { parent, name })
                }
                FileWriteMode::Replace {
                    expected_identity,
                    expected_sha256: _,
                } => {
                    let (path, metadata) = existing(&root, &request.path)?;
                    if identity(&metadata) != *expected_identity {
                        return Err(FileError::IdentityChanged);
                    }
                    (path, LeaseKey::Existing(*expected_identity))
                }
            };
            let _lease = acquire(&self.leases, lease_key)?;

            // 在取得进程内租约后重新获取宿主锁，避免两个Yonder调用并发跨过首次检查。
            let mut locked_target = match &request.mode {
                FileWriteMode::Replace {
                    expected_identity,
                    expected_sha256,
                } => Some(ensure_current(
                    &target,
                    *expected_identity,
                    expected_sha256,
                )?),
                FileWriteMode::CreateNew { .. } => None,
            };

            let parent = target.parent().ok_or(FileError::InvalidInput)?;
            if let FileWriteMode::CreateNew {
                expected_parent_identity,
            } = &request.mode
                && identity(&fs::metadata(parent).map_err(io_error)?) != *expected_parent_identity
            {
                return Err(FileError::IdentityChanged);
            }
            let (mut temporary, mut staged) = create_temporary(self, parent)?;
            if let Some(target_file) = &locked_target {
                staged
                    .set_permissions(target_file.metadata().map_err(io_error)?.permissions())
                    .map_err(io_error)?;
            }
            staged.write_all(&request.bytes).map_err(io_error)?;
            staged.sync_all().map_err(io_error)?;
            let staged_bytes = bounded_read(&mut staged)?;
            if staged_bytes != request.bytes || !validator.validate(&staged_bytes) {
                return Err(FileError::ValidationFailed);
            }

            match &request.mode {
                FileWriteMode::CreateNew {
                    expected_parent_identity,
                } => {
                    if identity(&fs::metadata(parent).map_err(io_error)?)
                        != *expected_parent_identity
                    {
                        return Err(FileError::IdentityChanged);
                    }
                    if fs::symlink_metadata(&target).is_ok() {
                        return Err(FileError::AlreadyExists);
                    }
                    drop(staged);
                    rename_exclusive(&temporary.path, &target)?;
                }
                FileWriteMode::Replace {
                    expected_identity,
                    expected_sha256,
                } => {
                    let current = locked_target.as_mut().ok_or(FileError::Unknown)?;
                    let metadata = current.metadata().map_err(io_error)?;
                    if identity(&metadata) != *expected_identity {
                        return Err(FileError::IdentityChanged);
                    }
                    let bytes = bounded_read(current)?;
                    if hash(&bytes) != *expected_sha256 {
                        return Err(FileError::ContentChanged);
                    }
                    drop(staged);
                    fs::rename(&temporary.path, &target).map_err(io_error)?;
                }
            }
            temporary.committed = true;
            #[cfg(test)]
            if self.fail_parent_sync {
                return Err(FileError::Unknown);
            }
            File::open(parent)
                .and_then(|directory| directory.sync_all())
                .map_err(|_| FileError::Unknown)?;
            drop(locked_target);

            let canonical = fs::canonicalize(&target).map_err(|_| FileError::Unknown)?;
            let metadata = fs::metadata(&canonical).map_err(|_| FileError::Unknown)?;
            if !metadata.is_file() || metadata.len() != request.bytes.len() as u64 {
                return Err(FileError::Unknown);
            }
            Ok(FileWriteReceipt {
                canonical_path: canonical
                    .to_str()
                    .ok_or(FileError::InvalidInput)?
                    .to_owned(),
                identity: identity(&metadata),
                sha256: hash(&request.bytes),
                bytes_written: request.bytes.len() as u64,
            })
        }

        fn write_atomic_guarded(
            &self,
            source: &FileSourceGuard,
            request: &FileWriteRequest,
            validator: &dyn FileValidator,
        ) -> Result<FileWriteReceipt, FileError> {
            validate_location(&source.source.path, &source.source.authorized_root)?;
            validate_location(&request.path, &request.authorized_root)?;
            if !valid_hash(&source.expected_sha256)
                || matches!(
                    &request.mode,
                    FileWriteMode::Replace {
                        expected_sha256,
                        ..
                    } if !valid_hash(expected_sha256)
                )
            {
                return Err(FileError::InvalidInput);
            }
            if request.bytes.len() > MAX_FILE_CONTENT_BYTES {
                return Err(FileError::TooLarge);
            }

            let root = canonical_root(&source.source.authorized_root)?;
            let (source_path, metadata) = existing(&root, &source.source.path)?;
            if identity(&metadata) != source.expected_identity {
                return Err(FileError::IdentityChanged);
            }
            let _source_lease =
                acquire(&self.leases, LeaseKey::Existing(source.expected_identity))?;
            let mut source_file = OpenOptions::new()
                .read(true)
                .open(&source_path)
                .map_err(io_error)?;
            FileExt::try_lock_shared(&source_file).map_err(|_| FileError::HostLocked)?;
            verify_source(
                &mut source_file,
                &source.source.path,
                &root,
                source.expected_identity,
                &source.expected_sha256,
            )?;

            let guarded = GuardedValidator {
                file: Mutex::new(source_file),
                requested_path: source.source.path.clone(),
                root,
                expected_identity: source.expected_identity,
                expected_hash: source.expected_sha256.clone(),
                inner: validator,
                error: Mutex::new(None),
            };
            let receipt = match self.write_atomic(request, &guarded) {
                Err(FileError::ValidationFailed) => {
                    return Err(guarded.take_error().unwrap_or(FileError::ValidationFailed));
                }
                result => result?,
            };
            if guarded.verify().is_err() {
                return Err(FileError::Unknown);
            }
            Ok(receipt)
        }

        fn trash(
            &self,
            request: &FileTrashRequest,
            _: &LocalTrashAuthorization,
        ) -> Result<FileTrashReceipt, FileError> {
            trash_impl(self, request).map(|result| result.0)
        }
    }
}

#[cfg(all(test, target_os = "macos"))]
mod tests {
    use super::*;
    use fs2::FileExt;
    use std::{
        fs::{self, OpenOptions},
        os::unix::fs::{MetadataExt, symlink},
        path::{Path, PathBuf},
        time::{SystemTime, UNIX_EPOCH},
    };
    use yonder_application::{
        AuthContext,
        file::{
            AcceptAnyFile, FileIdentity, FileSourceGuard, FileWriteMode, MAX_FILE_CONTENT_BYTES,
            inspect_create_target, read, trash, write_atomic, write_atomic_guarded,
        },
    };

    struct TestRoot(PathBuf);

    impl TestRoot {
        fn new(label: &str) -> Self {
            let unique = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos();
            let path = std::env::temp_dir().join(format!(
                "yonder-file-{label}-{}-{unique}",
                std::process::id()
            ));
            fs::create_dir(&path).unwrap();
            Self(path)
        }

        fn value(&self) -> String {
            self.0.to_str().unwrap().to_owned()
        }
    }

    impl Drop for TestRoot {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    fn value(path: &Path) -> String {
        path.to_str().unwrap().to_owned()
    }

    fn read_file(adapter: &ControlledFileAdapter, root: &TestRoot, path: &Path) -> FileSnapshot {
        read(
            adapter,
            &FileReadRequest {
                path: value(path),
                authorized_root: root.value(),
            },
        )
        .unwrap()
    }

    fn replace(
        root: &TestRoot,
        path: &Path,
        snapshot: &FileSnapshot,
        bytes: &[u8],
    ) -> FileWriteRequest {
        FileWriteRequest {
            path: value(path),
            authorized_root: root.value(),
            bytes: bytes.to_vec(),
            mode: FileWriteMode::Replace {
                expected_identity: snapshot.identity,
                expected_sha256: snapshot.sha256.clone(),
            },
        }
    }

    fn create_new(root: &TestRoot) -> FileWriteMode {
        let metadata = fs::metadata(fs::canonicalize(&root.0).unwrap()).unwrap();
        FileWriteMode::CreateNew {
            expected_parent_identity: FileIdentity {
                volume_id: metadata.dev(),
                file_id: metadata.ino(),
            },
        }
    }

    struct Reject;

    impl FileValidator for Reject {
        fn validate(&self, _: &[u8]) -> bool {
            false
        }
    }

    struct CreateRace(PathBuf);

    impl FileValidator for CreateRace {
        fn validate(&self, _: &[u8]) -> bool {
            fs::write(&self.0, b"intruder").unwrap();
            true
        }
    }

    struct ChangeSource(PathBuf);

    impl FileValidator for ChangeSource {
        fn validate(&self, _: &[u8]) -> bool {
            fs::write(&self.0, b"changed while validating").unwrap();
            true
        }
    }

    #[test]
    fn aliases_share_identity_and_escape_or_oversize_is_rejected() {
        let root = TestRoot::new("identity");
        let outside = TestRoot::new("outside");
        let source = root.0.join("source.txt");
        let hard = root.0.join("hard.txt");
        let soft = root.0.join("soft.txt");
        fs::write(&source, b"hello").unwrap();
        fs::hard_link(&source, &hard).unwrap();
        symlink(&source, &soft).unwrap();
        let adapter = ControlledFileAdapter::default();

        let source_snapshot = read_file(&adapter, &root, &source);
        assert_eq!(
            read_file(&adapter, &root, &hard).identity,
            source_snapshot.identity
        );
        assert_eq!(
            read_file(&adapter, &root, &soft).identity,
            source_snapshot.identity
        );

        let escape = root.0.join("escape");
        symlink(&outside.0, &escape).unwrap();
        fs::write(outside.0.join("private.txt"), b"private").unwrap();
        assert_eq!(
            read(
                &adapter,
                &FileReadRequest {
                    path: value(&escape.join("private.txt")),
                    authorized_root: root.value(),
                }
            ),
            Err(FileError::OutsideAuthorizedRoot)
        );

        let large = root.0.join("large.bin");
        let large_file = fs::File::create(&large).unwrap();
        large_file
            .set_len((MAX_FILE_CONTENT_BYTES + 1) as u64)
            .unwrap();
        assert_eq!(
            read(
                &adapter,
                &FileReadRequest {
                    path: value(&large),
                    authorized_root: root.value(),
                }
            ),
            Err(FileError::TooLarge)
        );
        assert_eq!(
            adapter.read(&FileReadRequest {
                path: "relative.txt".into(),
                authorized_root: root.value(),
            }),
            Err(FileError::InvalidInput)
        );
        assert_eq!(
            adapter.write_atomic(
                &FileWriteRequest {
                    path: value(&root.0.join("too-large.bin")),
                    authorized_root: root.value(),
                    bytes: vec![0; MAX_FILE_CONTENT_BYTES + 1],
                    mode: create_new(&root),
                },
                &AcceptAnyFile,
            ),
            Err(FileError::TooLarge)
        );

        let target = root.0.join("new-target.txt");
        let inspected = inspect_create_target(
            &adapter,
            &FileCreateTargetRequest {
                path: value(&target),
                authorized_root: root.value(),
            },
        )
        .unwrap();
        let canonical_root = fs::canonicalize(&root.0).unwrap();
        assert_eq!(
            inspected.canonical_path,
            value(&canonical_root.join("new-target.txt"))
        );
        assert_eq!(
            inspected.parent_identity,
            FileIdentity {
                volume_id: fs::metadata(&canonical_root).unwrap().dev(),
                file_id: fs::metadata(&canonical_root).unwrap().ino(),
            }
        );
        assert!(!target.exists());
        assert_eq!(
            inspect_create_target(
                &adapter,
                &FileCreateTargetRequest {
                    path: value(&escape.join("new.txt")),
                    authorized_root: root.value(),
                },
            ),
            Err(FileError::OutsideAuthorizedRoot)
        );
        assert_eq!(
            inspect_create_target(
                &adapter,
                &FileCreateTargetRequest {
                    path: value(&source),
                    authorized_root: root.value(),
                },
            ),
            Err(FileError::AlreadyExists)
        );
    }

    #[test]
    fn create_replace_validation_and_race_preserve_the_committed_target() {
        let root = TestRoot::new("atomic");
        let adapter = ControlledFileAdapter::default();
        let target = root.0.join("created.txt");
        let create = FileWriteRequest {
            path: value(&target),
            authorized_root: root.value(),
            bytes: b"first".to_vec(),
            mode: create_new(&root),
        };
        let receipt = write_atomic(&adapter, &create, &AcceptAnyFile).unwrap();
        assert_eq!(receipt.bytes_written, 5);
        assert_eq!(fs::read(&target).unwrap(), b"first");

        let wrong_parent_target = root.0.join("wrong-parent.txt");
        assert_eq!(
            write_atomic(
                &adapter,
                &FileWriteRequest {
                    path: value(&wrong_parent_target),
                    authorized_root: root.value(),
                    bytes: b"must not be written".to_vec(),
                    mode: FileWriteMode::CreateNew {
                        expected_parent_identity: FileIdentity {
                            volume_id: 0,
                            file_id: 0,
                        },
                    },
                },
                &AcceptAnyFile,
            ),
            Err(FileError::IdentityChanged)
        );
        assert!(!wrong_parent_target.exists());
        assert_eq!(
            write_atomic(&adapter, &create, &AcceptAnyFile),
            Err(FileError::AlreadyExists)
        );

        let first = read_file(&adapter, &root, &target);
        let replaced = write_atomic(
            &adapter,
            &replace(&root, &target, &first, b"second"),
            &AcceptAnyFile,
        )
        .unwrap();
        assert_ne!(replaced.identity, first.identity);
        assert_eq!(fs::read(&target).unwrap(), b"second");
        assert_eq!(
            write_atomic(
                &adapter,
                &replace(&root, &target, &first, b"stale"),
                &AcceptAnyFile
            ),
            Err(FileError::IdentityChanged)
        );

        let second = read_file(&adapter, &root, &target);
        assert_eq!(
            write_atomic(
                &adapter,
                &replace(&root, &target, &second, b"rejected"),
                &Reject
            ),
            Err(FileError::ValidationFailed)
        );
        assert_eq!(fs::read(&target).unwrap(), b"second");

        fs::write(&target, b"changed-in-place").unwrap();
        assert_eq!(
            write_atomic(
                &adapter,
                &replace(&root, &target, &second, b"third"),
                &AcceptAnyFile
            ),
            Err(FileError::ContentChanged)
        );
        assert_eq!(fs::read(&target).unwrap(), b"changed-in-place");

        let raced = root.0.join("raced.txt");
        let request = FileWriteRequest {
            path: value(&raced),
            authorized_root: root.value(),
            bytes: b"ours".to_vec(),
            mode: create_new(&root),
        };
        assert_eq!(
            write_atomic(&adapter, &request, &CreateRace(raced.clone())),
            Err(FileError::AlreadyExists)
        );
        assert_eq!(fs::read(&raced).unwrap(), b"intruder");
        assert!(fs::read_dir(&root.0).unwrap().all(|entry| {
            !entry
                .unwrap()
                .file_name()
                .to_string_lossy()
                .starts_with(".yonder-")
        }));

        let identity_source = root.0.join("identity-source.txt");
        let replacement = root.0.join("replacement.txt");
        let identity_target = root.0.join("identity-target.txt");
        fs::write(&identity_source, b"original identity").unwrap();
        let identity_snapshot = read_file(&adapter, &root, &identity_source);
        fs::write(&replacement, b"replacement identity").unwrap();
        fs::rename(&replacement, &identity_source).unwrap();
        assert_eq!(
            write_atomic_guarded(
                &adapter,
                &FileSourceGuard {
                    source: FileReadRequest {
                        path: value(&identity_source),
                        authorized_root: root.value(),
                    },
                    expected_identity: identity_snapshot.identity,
                    expected_sha256: identity_snapshot.sha256,
                },
                &FileWriteRequest {
                    path: value(&identity_target),
                    authorized_root: root.value(),
                    bytes: b"output".to_vec(),
                    mode: create_new(&root),
                },
                &AcceptAnyFile,
            ),
            Err(FileError::IdentityChanged)
        );
        assert!(!identity_target.exists());
    }

    #[test]
    fn guarded_create_rejects_source_change_without_publishing_output() {
        let root = TestRoot::new("guarded");
        let source = root.0.join("source.txt");
        let target = root.0.join("target.txt");
        fs::write(&source, b"before").unwrap();
        let adapter = ControlledFileAdapter::default();
        let snapshot = read_file(&adapter, &root, &source);
        let source_request = FileReadRequest {
            path: value(&source),
            authorized_root: root.value(),
        };
        let request = FileWriteRequest {
            path: value(&target),
            authorized_root: root.value(),
            bytes: b"output".to_vec(),
            mode: create_new(&root),
        };
        assert_eq!(
            write_atomic_guarded(
                &adapter,
                &FileSourceGuard {
                    source: source_request,
                    expected_identity: snapshot.identity,
                    expected_sha256: snapshot.sha256,
                },
                &request,
                &ChangeSource(source.clone()),
            ),
            Err(FileError::ContentChanged)
        );
        assert!(!target.exists());
        assert_eq!(fs::read(source).unwrap(), b"changed while validating");
        assert!(fs::read_dir(&root.0).unwrap().all(|entry| {
            !entry
                .unwrap()
                .file_name()
                .to_string_lossy()
                .starts_with(".yonder-")
        }));
    }

    #[test]
    fn aliases_cannot_take_a_second_lease_and_host_lock_is_distinct() {
        let root = TestRoot::new("locks");
        let source = root.0.join("source.txt");
        let hard = root.0.join("hard.txt");
        fs::write(&source, b"before").unwrap();
        fs::hard_link(&source, &hard).unwrap();
        let adapter = ControlledFileAdapter::default();
        let snapshot = read_file(&adapter, &root, &source);
        assert_eq!(
            read_file(&adapter, &root, &hard).identity,
            snapshot.identity
        );
        let lease = macos::acquire(
            &adapter.leases,
            macos::LeaseKey::Existing(snapshot.identity),
        )
        .unwrap();
        assert_eq!(
            macos::acquire(
                &adapter.leases,
                macos::LeaseKey::Existing(snapshot.identity)
            )
            .err(),
            Some(FileError::Busy)
        );
        drop(lease);

        let current = read_file(&adapter, &root, &source);
        let external = OpenOptions::new()
            .read(true)
            .write(true)
            .open(&source)
            .unwrap();
        FileExt::try_lock_exclusive(&external).unwrap();
        assert_eq!(
            write_atomic(
                &adapter,
                &replace(&root, &source, &current, b"blocked"),
                &AcceptAnyFile
            ),
            Err(FileError::HostLocked)
        );
        FileExt::unlock(&external).unwrap();
    }

    #[test]
    fn post_rename_sync_failure_is_unknown_and_native_trash_is_recoverable() {
        let root = TestRoot::new("unknown-trash");
        let target = root.0.join("unknown.txt");
        let failing = ControlledFileAdapter {
            fail_parent_sync: true,
            ..ControlledFileAdapter::default()
        };
        let request = FileWriteRequest {
            path: value(&target),
            authorized_root: root.value(),
            bytes: b"possibly-committed".to_vec(),
            mode: create_new(&root),
        };
        assert_eq!(
            write_atomic(&failing, &request, &AcceptAnyFile),
            Err(FileError::Unknown)
        );
        assert_eq!(fs::read(&target).unwrap(), b"possibly-committed");

        let adapter = ControlledFileAdapter::default();
        let trash_target = root.0.join("trash.txt");
        fs::write(&trash_target, b"trash fixture").unwrap();
        let snapshot = read_file(&adapter, &root, &trash_target);
        let request = FileTrashRequest {
            path: value(&trash_target),
            authorized_root: root.value(),
            expected_identity: snapshot.identity,
        };
        assert_eq!(
            trash(&adapter, AuthContext::Agent("agent-a"), &request),
            Err(FileError::PermissionDenied)
        );
        assert!(trash_target.exists());
        let (receipt, moved) = macos::trash_impl(&adapter, &request).unwrap();
        assert_eq!(receipt.identity, snapshot.identity);
        assert!(!trash_target.exists() && moved.exists());
        fs::remove_file(moved).unwrap();
    }

    #[test]
    fn stale_identity_is_rejected_before_native_trash() {
        let root = TestRoot::new("trash-stale");
        let target = root.0.join("target.txt");
        fs::write(&target, b"keep").unwrap();
        let adapter = ControlledFileAdapter::default();
        let request = FileTrashRequest {
            path: value(&target),
            authorized_root: root.value(),
            expected_identity: FileIdentity {
                volume_id: 0,
                file_id: 0,
            },
        };
        assert_eq!(
            trash(&adapter, AuthContext::LocalUser("desktop"), &request),
            Err(FileError::IdentityChanged)
        );
        assert_eq!(fs::read(target).unwrap(), b"keep");
    }
}
