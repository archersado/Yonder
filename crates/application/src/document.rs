use crate::{
    AuthContext,
    file::{
        FileError, FileIdentity, FilePort, FileReadRequest, FileSourceGuard, FileValidator,
        FileWriteMode, FileWriteRequest,
    },
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DocumentFormat {
    Docx,
    Xlsx,
    Pptx,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DocumentSnapshot {
    pub format: DocumentFormat,
    pub sha256: String,
    pub texts: Vec<String>,
    pub truncated: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReplaceUniqueText<'a> {
    pub before: &'a str,
    pub after: &'a str,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DocumentTransform {
    pub format: DocumentFormat,
    pub source_sha256: String,
    pub output_sha256: String,
    pub bytes: Vec<u8>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DocumentError {
    InvalidInput,
    UnsupportedFormat,
    InvalidArchive,
    InvalidXml,
    LimitExceeded,
    TargetNotFound,
    TargetAmbiguous,
}

pub trait DocumentPort: Send + Sync {
    fn inspect(
        &self,
        source: &[u8],
        max_text_nodes: usize,
    ) -> Result<DocumentSnapshot, DocumentError>;
    fn replace_unique_text(
        &self,
        source: &[u8],
        operation: ReplaceUniqueText<'_>,
    ) -> Result<DocumentTransform, DocumentError>;
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DocumentFileSnapshot {
    pub canonical_path: String,
    pub identity: FileIdentity,
    pub document: DocumentSnapshot,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DocumentSaveAsRequest {
    pub source: FileReadRequest,
    pub output_path: String,
    pub output_authorized_root: String,
    pub expected_sha256: String,
    pub before: String,
    pub after: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DocumentOverwriteRequest {
    pub source: FileReadRequest,
    pub expected_sha256: String,
    pub before: String,
    pub after: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DocumentFileReceipt {
    pub format: DocumentFormat,
    pub canonical_path: String,
    pub identity: FileIdentity,
    pub sha256: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DocumentFileError {
    Document(DocumentError),
    File(FileError),
    HashConflict,
    PermissionDenied,
    Unknown,
}

impl From<DocumentError> for DocumentFileError {
    fn from(error: DocumentError) -> Self {
        Self::Document(error)
    }
}

impl From<FileError> for DocumentFileError {
    fn from(error: FileError) -> Self {
        Self::File(error)
    }
}

struct OoxmlValidator<'a> {
    documents: &'a dyn DocumentPort,
    expected_format: DocumentFormat,
}

impl FileValidator for OoxmlValidator<'_> {
    fn validate(&self, staged_bytes: &[u8]) -> bool {
        self.documents
            .inspect(staged_bytes, 1)
            .is_ok_and(|snapshot| snapshot.format == self.expected_format)
    }
}

pub fn inspect_file(
    files: &dyn FilePort,
    documents: &dyn DocumentPort,
    request: &FileReadRequest,
    max_text_nodes: usize,
) -> Result<DocumentFileSnapshot, DocumentFileError> {
    let source = crate::file::read(files, request)?;
    let document = documents.inspect(&source.bytes, max_text_nodes)?;
    if document.sha256 != source.sha256 {
        return Err(DocumentFileError::Unknown);
    }
    Ok(DocumentFileSnapshot {
        canonical_path: source.canonical_path,
        identity: source.identity,
        document,
    })
}

pub fn save_as(
    files: &dyn FilePort,
    documents: &dyn DocumentPort,
    request: &DocumentSaveAsRequest,
) -> Result<DocumentFileReceipt, DocumentFileError> {
    let source = crate::file::read(files, &request.source)?;
    if source.sha256 != request.expected_sha256 {
        return Err(DocumentFileError::HashConflict);
    }
    let transform = documents.replace_unique_text(
        &source.bytes,
        ReplaceUniqueText {
            before: &request.before,
            after: &request.after,
        },
    )?;
    if transform.source_sha256 != source.sha256 {
        return Err(DocumentFileError::Unknown);
    }
    let validator = OoxmlValidator {
        documents,
        expected_format: transform.format,
    };
    let receipt = crate::file::write_atomic_guarded(
        files,
        &FileSourceGuard {
            source: request.source.clone(),
            expected_identity: source.identity,
            expected_sha256: source.sha256,
        },
        &FileWriteRequest {
            path: request.output_path.clone(),
            authorized_root: request.output_authorized_root.clone(),
            bytes: transform.bytes,
            mode: FileWriteMode::CreateNew,
        },
        &validator,
    )?;
    receipt_from_transform(transform.format, transform.output_sha256, receipt)
}

pub fn overwrite_local(
    files: &dyn FilePort,
    documents: &dyn DocumentPort,
    auth: AuthContext<'_>,
    request: &DocumentOverwriteRequest,
) -> Result<DocumentFileReceipt, DocumentFileError> {
    if !matches!(auth, AuthContext::LocalUser(_)) {
        return Err(DocumentFileError::PermissionDenied);
    }
    let source = crate::file::read(files, &request.source)?;
    if source.sha256 != request.expected_sha256 {
        return Err(DocumentFileError::HashConflict);
    }
    let transform = documents.replace_unique_text(
        &source.bytes,
        ReplaceUniqueText {
            before: &request.before,
            after: &request.after,
        },
    )?;
    if transform.source_sha256 != source.sha256 {
        return Err(DocumentFileError::Unknown);
    }
    let validator = OoxmlValidator {
        documents,
        expected_format: transform.format,
    };
    let receipt = crate::file::write_atomic(
        files,
        &FileWriteRequest {
            path: source.canonical_path,
            authorized_root: request.source.authorized_root.clone(),
            bytes: transform.bytes,
            mode: FileWriteMode::Replace {
                expected_identity: source.identity,
                expected_sha256: source.sha256,
            },
        },
        &validator,
    )?;
    receipt_from_transform(transform.format, transform.output_sha256, receipt)
}

fn receipt_from_transform(
    format: DocumentFormat,
    output_sha256: String,
    receipt: crate::file::FileWriteReceipt,
) -> Result<DocumentFileReceipt, DocumentFileError> {
    if receipt.sha256 != output_sha256 {
        return Err(DocumentFileError::Unknown);
    }
    Ok(DocumentFileReceipt {
        format,
        canonical_path: receipt.canonical_path,
        identity: receipt.identity,
        sha256: receipt.sha256,
    })
}
