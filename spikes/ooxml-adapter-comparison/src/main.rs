use quick_xml::{events::Event, reader::Reader, writer::Writer};
use sha2::{Digest, Sha256};
use std::{collections::BTreeMap, fs, io::{Cursor, Read, Write}, path::Path, time::Instant};
use zip::{CompressionMethod, ZipArchive, ZipWriter, write::SimpleFileOptions};

const CASES: [(&str, &str, &str, &str); 3] = [
    ("sample.docx", "word/document.xml", "YONDER_DOCX_BEFORE", "YONDER_DOCX_AFTER"),
    ("sample.xlsx", "xl/worksheets/sheet1.xml", "YONDER_XLSX_BEFORE", "YONDER_XLSX_AFTER"),
    ("sample.pptx", "ppt/slides/slide1.xml", "YONDER_PPTX_BEFORE", "YONDER_PPTX_AFTER"),
];
const REAL_CASES: [(&str, &str, &str, &str); 3] = [
    ("sample.docx", "word/document.xml", "Basic Integration Printer Test", "Yonder Integration Test"),
    ("sample.xlsx", "xl/sharedStrings.xml", "file name", "Yonder file name"),
    ("sample.pptx", "ppt/slides/slide3.xml", "Graphics Usage", "Yonder Graphics Usage"),
];

fn hash(bytes: &[u8]) -> String { format!("{:x}", Sha256::digest(bytes)) }

fn replace_xml(source: &[u8], before: &str, after: &str) -> Result<(Vec<u8>, usize), String> {
    let mut reader = Reader::from_reader(source);
    let mut writer = Writer::new(Vec::new());
    let mut replacements = 0;
    loop {
        match reader.read_event().map_err(|error| error.to_string())? {
            Event::Text(text) if text.xml10_content().contains(before) => {
                let value = text.xml10_content().replace(before, after);
                writer.write_event(Event::Text(quick_xml::events::BytesText::new(&value))).map_err(|error| error.to_string())?;
                replacements += 1;
            }
            Event::Eof => break,
            event => writer.write_event(event).map_err(|error| error.to_string())?,
        }
    }
    Ok((writer.into_inner(), replacements))
}

fn transform(input: &Path, output: &Path, expected_hash: &str, part: &str, before: &str, after: &str) -> Result<(usize, usize), String> {
    let source = fs::read(input).map_err(|error| error.to_string())?;
    if hash(&source) != expected_hash { return Err("document_conflict".into()); }
    let mut archive = ZipArchive::new(Cursor::new(source)).map_err(|error| error.to_string())?;
    let mut parts = BTreeMap::new();
    for index in 0..archive.len() {
        let mut entry = archive.by_index(index).map_err(|error| error.to_string())?;
        let mut bytes = Vec::new();
        entry.read_to_end(&mut bytes).map_err(|error| error.to_string())?;
        parts.insert(entry.name().to_owned(), bytes);
    }
    let original: BTreeMap<_, _> = parts.iter().map(|(name, bytes)| (name.clone(), hash(bytes))).collect();
    let (xml, replacements) = replace_xml(parts.get(part).ok_or("missing_part")?, before, after)?;
    if replacements != 1 { return Err(format!("replacement_count:{replacements}")); }
    parts.insert(part.into(), xml);

    let temporary = output.with_extension(format!("{}.tmp", std::process::id()));
    let file = fs::File::create(&temporary).map_err(|error| error.to_string())?;
    let mut zip = ZipWriter::new(file);
    let options = SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);
    for (name, bytes) in &parts {
        zip.start_file(name, options).map_err(|error| error.to_string())?;
        zip.write_all(bytes).map_err(|error| error.to_string())?;
    }
    zip.finish().map_err(|error| error.to_string())?;
    let written = fs::read(&temporary).map_err(|error| error.to_string())?;
    let mut check = ZipArchive::new(Cursor::new(&written)).map_err(|error| error.to_string())?;
    for (name, expected) in original {
        if name == part { continue; }
        let mut bytes = Vec::new();
        check.by_name(&name).map_err(|error| error.to_string())?.read_to_end(&mut bytes).map_err(|error| error.to_string())?;
        if hash(&bytes) != expected { return Err(format!("fidelity:{name}")); }
    }
    fs::rename(temporary, output).map_err(|error| error.to_string())?;
    Ok((replacements, written.len()))
}

fn main() -> Result<(), String> {
    fs::create_dir_all("output/rust").map_err(|error| error.to_string())?;
    let started = Instant::now();
    for (name, part, before, after) in CASES {
        let input = Path::new("fixtures/synthetic").join(name);
        let output = Path::new("output/rust").join(name);
        let source = fs::read(&input).map_err(|error| error.to_string())?;
        let (replacements, bytes) = transform(&input, &output, &hash(&source), part, before, after)?;
        println!(r#"{{"name":"{name}","replacements":{replacements},"bytes":{bytes}}}"#);
    }
    for (name, part, before, after) in REAL_CASES {
        let input = Path::new("fixtures/real").join(name);
        let output = Path::new("output/rust").join(format!("real-{name}"));
        let source = fs::read(&input).map_err(|error| error.to_string())?;
        let (replacements, bytes) = transform(&input, &output, &hash(&source), part, before, after)?;
        println!(r#"{{"name":"real-{name}","replacements":{replacements},"bytes":{bytes}}}"#);
    }
    let (name, part, before, after) = CASES[0];
    let conflict = transform(Path::new("fixtures/synthetic").join(name).as_path(), Path::new("output/rust/conflict.docx"), "bad-hash", part, before, after).unwrap_err() == "document_conflict";
    if !conflict { return Err("expected_hash 门禁失败".into()); }
    println!(r#"{{"conflict":true,"elapsed_ms":{}}}"#, started.elapsed().as_millis());
    Ok(())
}
