#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DocumentFormat { Docx, Xlsx, Pptx }

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DocumentSnapshot {
    pub format: DocumentFormat,
    pub sha256: String,
    pub texts: Vec<String>,
    pub truncated: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReplaceUniqueText<'a> { pub before: &'a str, pub after: &'a str }

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

pub trait DocumentPort {
    fn inspect(&self, source: &[u8], max_text_nodes: usize) -> Result<DocumentSnapshot, DocumentError>;
    fn replace_unique_text(&self, source: &[u8], operation: ReplaceUniqueText<'_>) -> Result<DocumentTransform, DocumentError>;
}
