use quick_xml::{events::{BytesText, Event}, Reader, Writer};
use sha2::{Digest, Sha256};
use std::{collections::HashSet, io::{Cursor, Read, Write}, path::{Component, Path}};
use yonder_application::document::{DocumentError, DocumentFormat, DocumentPort, DocumentSnapshot, DocumentTransform, ReplaceUniqueText};
use zip::{write::SimpleFileOptions, CompressionMethod, ZipArchive, ZipWriter};

const MAX_SOURCE_BYTES: usize = 64 * 1024 * 1024;
const MAX_ENTRY_BYTES: u64 = 32 * 1024 * 1024;
const MAX_TOTAL_BYTES: u64 = 256 * 1024 * 1024;
const MAX_ENTRIES: usize = 4096;
const MAX_TEXT_NODES: usize = 10_000;
const MAX_REPLACEMENT_BYTES: usize = 64 * 1024;

pub struct OoxmlDocumentAdapter;

struct Entry { name: String, bytes: Vec<u8>, compression: CompressionMethod, directory: bool, mode: Option<u32> }
struct Package { format: DocumentFormat, entries: Vec<Entry> }

fn hash(bytes: &[u8]) -> String { format!("{:x}", Sha256::digest(bytes)) }

fn safe_name(name: &str) -> bool {
    !name.contains('\\') && !Path::new(name).components().any(|part| matches!(part, Component::ParentDir | Component::RootDir | Component::Prefix(_)))
}

fn parse_package(source: &[u8]) -> Result<Package, DocumentError> {
    if source.is_empty() || source.len() > MAX_SOURCE_BYTES { return Err(DocumentError::LimitExceeded); }
    let mut zip = ZipArchive::new(Cursor::new(source)).map_err(|_| DocumentError::InvalidArchive)?;
    if zip.len() == 0 || zip.len() > MAX_ENTRIES { return Err(DocumentError::LimitExceeded); }
    let mut entries = Vec::with_capacity(zip.len());
    let mut names = HashSet::new();
    let mut total = 0_u64;
    for index in 0..zip.len() {
        let mut file = zip.by_index(index).map_err(|_| DocumentError::InvalidArchive)?;
        let name = file.name().to_owned();
        total = total.checked_add(file.size()).ok_or(DocumentError::LimitExceeded)?;
        if !safe_name(&name) || !names.insert(name.clone()) || file.size() > MAX_ENTRY_BYTES || total > MAX_TOTAL_BYTES { return Err(DocumentError::LimitExceeded); }
        let mut bytes = Vec::with_capacity(file.size() as usize);
        file.read_to_end(&mut bytes).map_err(|_| DocumentError::InvalidArchive)?;
        entries.push(Entry { name, bytes, compression: file.compression(), directory: file.is_dir(), mode: file.unix_mode() });
    }
    let has = |name: &str| entries.iter().any(|entry| entry.name == name);
    let formats = [(has("word/document.xml"), DocumentFormat::Docx), (has("xl/workbook.xml"), DocumentFormat::Xlsx), (has("ppt/presentation.xml"), DocumentFormat::Pptx)];
    let mut found = formats.into_iter().filter_map(|(present, format)| present.then_some(format));
    let format = found.next().ok_or(DocumentError::UnsupportedFormat)?;
    if found.next().is_some() { return Err(DocumentError::UnsupportedFormat); }
    Ok(Package { format, entries })
}

fn text_part(format: DocumentFormat, name: &str) -> bool {
    match format {
        DocumentFormat::Docx => name == "word/document.xml" || ((name.starts_with("word/header") || name.starts_with("word/footer")) && name.ends_with(".xml")),
        DocumentFormat::Xlsx => name == "xl/sharedStrings.xml" || (name.starts_with("xl/worksheets/") && name.ends_with(".xml")),
        DocumentFormat::Pptx => name.starts_with("ppt/slides/slide") && name.ends_with(".xml"),
    }
}

fn read_texts(xml: &[u8], texts: &mut Vec<String>, limit: usize, truncated: &mut bool) -> Result<(), DocumentError> {
    let mut reader = Reader::from_reader(xml);
    loop {
        match reader.read_event().map_err(|_| DocumentError::InvalidXml)? {
            Event::Text(text) => {
                let value = text.xml10_content();
                if !value.is_empty() {
                    if texts.len() < limit { texts.push(value.into_owned()); } else { *truncated = true; }
                }
            }
            Event::Eof => return Ok(()),
            _ => {}
        }
    }
}

fn occurrences(xml: &[u8], needle: &str) -> Result<usize, DocumentError> {
    let mut reader = Reader::from_reader(xml);
    let mut count = 0;
    loop {
        match reader.read_event().map_err(|_| DocumentError::InvalidXml)? {
            Event::Text(text) => count += text.xml10_content().matches(needle).count(),
            Event::Eof => return Ok(count),
            _ => {}
        }
    }
}

fn replace(xml: &[u8], before: &str, after: &str) -> Result<Vec<u8>, DocumentError> {
    let mut reader = Reader::from_reader(xml);
    let mut writer = Writer::new(Vec::with_capacity(xml.len()));
    loop {
        match reader.read_event().map_err(|_| DocumentError::InvalidXml)? {
            Event::Text(text) if text.xml10_content().contains(before) => {
                let value = text.xml10_content().replace(before, after);
                writer.write_event(Event::Text(BytesText::new(&value))).map_err(|_| DocumentError::InvalidXml)?;
            }
            Event::Eof => return Ok(writer.into_inner()),
            event => writer.write_event(event).map_err(|_| DocumentError::InvalidXml)?,
        }
    }
}

fn write_package(package: &Package) -> Result<Vec<u8>, DocumentError> {
    let mut zip = ZipWriter::new(Cursor::new(Vec::new()));
    for entry in &package.entries {
        let mut options = SimpleFileOptions::default().compression_method(entry.compression);
        if let Some(mode) = entry.mode { options = options.unix_permissions(mode); }
        if entry.directory { zip.add_directory(&entry.name, options) } else { zip.start_file(&entry.name, options) }.map_err(|_| DocumentError::InvalidArchive)?;
        if !entry.directory { zip.write_all(&entry.bytes).map_err(|_| DocumentError::InvalidArchive)?; }
    }
    Ok(zip.finish().map_err(|_| DocumentError::InvalidArchive)?.into_inner())
}

impl DocumentPort for OoxmlDocumentAdapter {
    fn inspect(&self, source: &[u8], max_text_nodes: usize) -> Result<DocumentSnapshot, DocumentError> {
        if max_text_nodes == 0 || max_text_nodes > MAX_TEXT_NODES { return Err(DocumentError::InvalidInput); }
        let package = parse_package(source)?;
        let mut texts = Vec::new();
        let mut truncated = false;
        for entry in package.entries.iter().filter(|entry| text_part(package.format, &entry.name)) { read_texts(&entry.bytes, &mut texts, max_text_nodes, &mut truncated)?; }
        Ok(DocumentSnapshot { format: package.format, sha256: hash(source), texts, truncated })
    }

    fn replace_unique_text(&self, source: &[u8], operation: ReplaceUniqueText<'_>) -> Result<DocumentTransform, DocumentError> {
        if operation.before.is_empty() || operation.before.len() > MAX_REPLACEMENT_BYTES || operation.after.len() > MAX_REPLACEMENT_BYTES { return Err(DocumentError::InvalidInput); }
        let mut package = parse_package(source)?;
        let mut count = 0;
        for entry in package.entries.iter().filter(|entry| text_part(package.format, &entry.name)) { count += occurrences(&entry.bytes, operation.before)?; }
        match count { 0 => return Err(DocumentError::TargetNotFound), 1 => {}, _ => return Err(DocumentError::TargetAmbiguous) }
        for entry in package.entries.iter_mut().filter(|entry| text_part(package.format, &entry.name)) {
            if occurrences(&entry.bytes, operation.before)? == 1 { entry.bytes = replace(&entry.bytes, operation.before, operation.after)?; break; }
        }
        let bytes = write_package(&package)?;
        if parse_package(&bytes)?.format != package.format { return Err(DocumentError::InvalidArchive); }
        Ok(DocumentTransform { format: package.format, source_sha256: hash(source), output_sha256: hash(&bytes), bytes })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const CASES: [(&[u8], DocumentFormat, &str, &str); 3] = [
        (include_bytes!("../../../spikes/ooxml-adapter-comparison/fixtures/synthetic/sample.docx"), DocumentFormat::Docx, "YONDER_DOCX_BEFORE", "YONDER_DOCX_AFTER"),
        (include_bytes!("../../../spikes/ooxml-adapter-comparison/fixtures/synthetic/sample.xlsx"), DocumentFormat::Xlsx, "YONDER_XLSX_BEFORE", "YONDER_XLSX_AFTER"),
        (include_bytes!("../../../spikes/ooxml-adapter-comparison/fixtures/synthetic/sample.pptx"), DocumentFormat::Pptx, "YONDER_PPTX_BEFORE", "YONDER_PPTX_AFTER"),
    ];

    #[test]
    fn reads_and_replaces_all_supported_formats() {
        let adapter = OoxmlDocumentAdapter;
        for (source, format, before, after) in CASES {
            let original = source.to_vec();
            let snapshot = adapter.inspect(source, 100).unwrap();
            assert_eq!(snapshot.format, format);
            assert!(snapshot.texts.iter().any(|text| text.contains(before)));
            let transformed = adapter.replace_unique_text(source, ReplaceUniqueText { before, after }).unwrap();
            assert_eq!(source, original);
            assert_eq!(transformed.format, format);
            assert!(adapter.inspect(&transformed.bytes, 100).unwrap().texts.iter().any(|text| text.contains(after)));
            let before_parts = parse_package(source).unwrap().entries.into_iter().map(|entry| (entry.name, entry.bytes)).collect::<std::collections::BTreeMap<_, _>>();
            let after_parts = parse_package(&transformed.bytes).unwrap().entries.into_iter().map(|entry| (entry.name, entry.bytes)).collect::<std::collections::BTreeMap<_, _>>();
            assert_eq!(before_parts.keys().collect::<Vec<_>>(), after_parts.keys().collect::<Vec<_>>());
            assert_eq!(before_parts.iter().filter(|(name, bytes)| after_parts.get(*name) != Some(*bytes)).count(), 1);
            assert_eq!(adapter.replace_unique_text(&transformed.bytes, ReplaceUniqueText { before: "missing", after }).unwrap_err(), DocumentError::TargetNotFound);
            let ambiguous = adapter.replace_unique_text(source, ReplaceUniqueText { before, after: "DUP DUP" }).unwrap();
            assert_eq!(adapter.replace_unique_text(&ambiguous.bytes, ReplaceUniqueText { before: "DUP", after }).unwrap_err(), DocumentError::TargetAmbiguous);
        }
    }
}
