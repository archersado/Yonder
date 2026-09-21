use image::ImageFormat;
use serde::Deserialize;
use std::{
    collections::{BTreeMap, HashSet},
    fs,
    io::{Cursor, Read},
    path::{Component, Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};
use zip::ZipArchive;

const MAX_SOURCE_BYTES: u64 = 64 * 1024 * 1024;
const MAX_MANIFEST_BYTES: usize = 1024 * 1024;
const MAX_ENTRY_BYTES: usize = 8 * 1024 * 1024;
const MAX_TOTAL_BYTES: usize = 32 * 1024 * 1024;
const MAX_ENTRIES: usize = 145;
const MAX_FRAMES_PER_STATE: usize = 16;
const MAX_TOTAL_PIXELS: u64 = 64 * 1024 * 1024;
const MAX_DIMENSION: u32 = 1024;
const STATES: [&str; 9] = [
    "idle", "listening", "recording", "thinking", "executing", "waiting_for_user", "success", "failed", "paused",
];

#[derive(Debug, Eq, PartialEq)]
pub enum PetPackError {
    Io,
    InvalidArchive,
    InvalidManifest,
    LimitExceeded,
    InvalidImage,
    SwitchFailed,
}

#[derive(Debug, Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
struct PackManifest {
    format_version: u8,
    states: BTreeMap<String, PackState>,
}

#[derive(Debug, Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
struct PackState {
    files: Vec<String>,
}

pub fn import_pet_pack(zip_path: &Path, packs_dir: &Path) -> Result<(), PetPackError> {
    let metadata = fs::metadata(zip_path).map_err(|_| PetPackError::Io)?;
    if !metadata.is_file() || metadata.len() > MAX_SOURCE_BYTES {
        return Err(PetPackError::LimitExceeded);
    }
    let source = fs::read(zip_path).map_err(|_| PetPackError::Io)?;
    let pack = validate(&source)?;
    let staging = staging_dir(packs_dir)?;
    if let Err(error) = write_pack(&pack, &staging) {
        let _ = fs::remove_dir_all(&staging);
        return Err(error);
    }
    if let Err(error) = activate(&staging, packs_dir) {
        let _ = fs::remove_dir_all(&staging);
        return Err(error);
    }
    Ok(())
}

pub fn import_pet_pack_bytes(source: &[u8], packs_dir: &Path) -> Result<(), PetPackError> {
    let pack = validate(source)?;
    let staging = staging_dir(packs_dir)?;
    if let Err(error) = write_pack(&pack, &staging) {
        let _ = fs::remove_dir_all(&staging);
        return Err(error);
    }
    if let Err(error) = activate(&staging, packs_dir) {
        let _ = fs::remove_dir_all(&staging);
        return Err(error);
    }
    Ok(())
}

#[derive(Debug, serde::Serialize)]
pub struct ActivePetPack {
    pub manifest: serde_json::Value,
    pub assets: Vec<(String, Vec<u8>)>,
}

pub fn active_pack(packs_dir: &Path) -> Result<ActivePetPack, PetPackError> {
    let manifest_path = packs_dir.join("current/manifest.json");
    let manifest_bytes = fs::read(&manifest_path).map_err(|_| PetPackError::Io)?;
    if manifest_bytes.len() > MAX_MANIFEST_BYTES { return Err(PetPackError::LimitExceeded); }
    let manifest: PackManifest = serde_json::from_slice(&manifest_bytes).map_err(|_| PetPackError::InvalidManifest)?;
    let mut assets = Vec::new();
    let mut names = HashSet::new();
    names.insert("manifest.json".to_owned());
    let mut total = 0_usize;
    let root = packs_dir.join("current");
    for entry in fs::read_dir(root.join("assets")).map_err(|_| PetPackError::Io)? {
        let entry = entry.map_err(|_| PetPackError::Io)?;
        if !entry.file_type().map_err(|_| PetPackError::Io)?.is_file() { return Err(PetPackError::InvalidManifest); }
        let name = entry.file_name().to_string_lossy().into_owned();
        let path = format!("assets/{name}");
        if !safe_file_name(&name) { return Err(PetPackError::InvalidManifest); }
        if !names.insert(path.clone()) { return Err(PetPackError::InvalidManifest); }
        let bytes = fs::read(entry.path()).map_err(|_| PetPackError::Io)?;
        if bytes.len() > MAX_ENTRY_BYTES || bytes.len().saturating_add(total) > MAX_TOTAL_BYTES {
            return Err(PetPackError::LimitExceeded);
        }
        validate_image(&bytes)?;
        total += bytes.len();
        assets.push((path, bytes));
    }
    validate_manifest(&manifest, &names)?;
    let manifest = serde_json::to_value(manifest).map_err(|_| PetPackError::InvalidManifest)?;
    Ok(ActivePetPack { manifest, assets })
}

fn validate(source: &[u8]) -> Result<Vec<(String, Vec<u8>)>, PetPackError> {
    let mut archive = ZipArchive::new(Cursor::new(source)).map_err(|_| PetPackError::InvalidArchive)?;
    if archive.is_empty() || archive.len() > MAX_ENTRIES {
        return Err(PetPackError::LimitExceeded);
    }
    let mut entries = Vec::new();
    let mut names = HashSet::new();
    let mut total = 0_usize;
    let mut total_pixels = 0_u64;
    for index in 0..archive.len() {
        let file = archive.by_index(index).map_err(|_| PetPackError::InvalidArchive)?;
        let name = file.name().to_owned();
        let expected = file.size() as usize;
        if file.is_dir()
            || file.unix_mode().is_some_and(|mode| mode & 0o170000 == 0o120000)
            || !safe_path(&name)
            || !names.insert(name.to_owned())
        {
            return Err(PetPackError::InvalidManifest);
        }
        if name != "manifest.json" && !(name.starts_with("assets/") && !name["assets/".len()..].contains('/')) {
            return Err(PetPackError::InvalidManifest);
        }
        let mut bytes = Vec::new();
        let read = file.take(MAX_ENTRY_BYTES as u64 + 1).read_to_end(&mut bytes).map_err(|_| PetPackError::InvalidArchive)?;
        if read != expected {
            return Err(PetPackError::InvalidArchive);
        }
        let entry_limit = if name == "manifest.json" { MAX_MANIFEST_BYTES } else { MAX_ENTRY_BYTES };
        if bytes.len() > entry_limit || bytes.len().saturating_add(total) > MAX_TOTAL_BYTES {
            return Err(PetPackError::LimitExceeded);
        }
        if name != "manifest.json" {
            let (width, height) = validate_image(&bytes)?;
            total_pixels = total_pixels.saturating_add(u64::from(width) * u64::from(height));
        }
        total += bytes.len();
        entries.push((name.to_owned(), bytes));
    }
    let manifest = entries
        .iter()
        .find_map(|(name, bytes)| (name == "manifest.json").then(|| bytes.as_slice()))
        .ok_or(PetPackError::InvalidManifest)?;
    let manifest: PackManifest = serde_json::from_slice(manifest).map_err(|_| PetPackError::InvalidManifest)?;
    validate_manifest(&manifest, &names)?;
    if total_pixels > MAX_TOTAL_PIXELS {
        return Err(PetPackError::LimitExceeded);
    }
    Ok(entries)
}

fn validate_manifest(manifest: &PackManifest, names: &HashSet<String>) -> Result<(), PetPackError> {
    if manifest.format_version != 1 || manifest.states.len() != STATES.len() || !STATES.iter().all(|state| manifest.states.contains_key(*state)) {
        return Err(PetPackError::InvalidManifest);
    }
    for state in manifest.states.values() {
        if state.files.is_empty() || state.files.len() > MAX_FRAMES_PER_STATE { return Err(PetPackError::InvalidManifest); }
        let mut frames = HashSet::new();
        for file in &state.files {
            let path = format!("assets/{file}");
            if !safe_file_name(file) || !frames.insert(file) || !names.contains(&path) {
                return Err(PetPackError::InvalidManifest);
            }
        }
    }
    if names.len() != 1 + manifest.states.values().map(|state| state.files.len()).sum::<usize>() {
        return Err(PetPackError::InvalidManifest);
    }
    Ok(())
}

fn safe_path(name: &str) -> bool {
    !name.contains('\\')
        && !Path::new(name).components().any(|part| matches!(part, Component::ParentDir | Component::RootDir | Component::Prefix(_)))
}

fn safe_file_name(name: &str) -> bool {
    safe_path(name) && !name.contains('/') && name != "." && name != ".."
}

fn validate_image(bytes: &[u8]) -> Result<(u32, u32), PetPackError> {
    let format = image::guess_format(bytes).map_err(|_| PetPackError::InvalidImage)?;
    if !matches!(format, ImageFormat::Png | ImageFormat::WebP) {
        return Err(PetPackError::InvalidImage);
    }
    let reader = image::ImageReader::new(Cursor::new(bytes)).with_guessed_format().map_err(|_| PetPackError::InvalidImage)?;
    let frame = reader.decode().map_err(|_| PetPackError::InvalidImage)?;
    if frame.width() == 0 || frame.height() == 0 || frame.width() > MAX_DIMENSION || frame.height() > MAX_DIMENSION {
        return Err(PetPackError::LimitExceeded);
    }
    Ok((frame.width(), frame.height()))
}

fn staging_dir(packs_dir: &Path) -> Result<PathBuf, PetPackError> {
    fs::create_dir_all(packs_dir).map_err(|_| PetPackError::Io)?;
    for entry in fs::read_dir(packs_dir).map_err(|_| PetPackError::Io)? {
        let entry = entry.map_err(|_| PetPackError::Io)?;
        if entry.file_name().to_string_lossy().starts_with(".import-") {
            let _ = fs::remove_dir_all(entry.path());
        }
    }
    let nanos = SystemTime::now().duration_since(UNIX_EPOCH).map_err(|_| PetPackError::Io)?.as_nanos();
    let path = packs_dir.join(format!(".import-{nanos}-{}", std::process::id()));
    fs::create_dir(&path).map_err(|_| PetPackError::Io)?;
    Ok(path)
}

fn write_pack(pack: &[(String, Vec<u8>)], staging: &Path) -> Result<(), PetPackError> {
    fs::create_dir_all(staging.join("assets")).map_err(|_| PetPackError::Io)?;
    for (name, bytes) in pack {
        fs::write(staging.join(name), bytes).map_err(|_| PetPackError::Io)?;
    }
    Ok(())
}

fn activate(staging: &Path, packs_dir: &Path) -> Result<(), PetPackError> {
    let current = packs_dir.join("current");
    if !current.exists() {
        let backup = packs_dir.join(".backup-current");
        if backup.exists() {
            fs::rename(&backup, &current).map_err(|_| PetPackError::SwitchFailed)?;
        }
        return fs::rename(staging, &current).map_err(|_| PetPackError::SwitchFailed);
    }
    let backup = packs_dir.join(".backup-current");
    let _ = fs::remove_dir_all(&backup);
    fs::rename(&current, &backup).map_err(|_| PetPackError::SwitchFailed)?;
    if let Err(error) = fs::rename(staging, &current) {
        let _ = fs::rename(&backup, &current);
        return Err(error).map_err(|_| PetPackError::SwitchFailed);
    }
    let _ = fs::remove_dir_all(&backup);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::{codecs::png::PngEncoder, codecs::webp::WebPEncoder, ExtendedColorType, ImageBuffer, ImageEncoder, LumaA};
    use std::io::Write;
    use zip::{write::SimpleFileOptions, ZipWriter};

    fn png() -> Vec<u8> {
        let image: ImageBuffer<LumaA<u8>, Vec<u8>> = ImageBuffer::from_pixel(2, 2, LumaA([0, 255]));
        let mut bytes = Vec::new();
        PngEncoder::new(&mut bytes).write_image(image.as_raw(), 2, 2, ExtendedColorType::La8).unwrap();
        bytes
    }

    fn webp() -> Vec<u8> {
        let image: ImageBuffer<LumaA<u8>, Vec<u8>> = ImageBuffer::from_pixel(2, 2, LumaA([0, 255]));
        let mut bytes = Vec::new();
        WebPEncoder::new_lossless(&mut bytes).write_image(image.as_raw(), 2, 2, ExtendedColorType::La8).unwrap();
        bytes
    }

    fn manifest() -> String {
        let frames: Vec<_> = STATES.iter().map(|state| format!("\"{state}\":{{\"files\":[\"{state}-00.png\"]}}")).collect();
        format!("{{\"format_version\":1,\"states\":{{{}}}}}", frames.join(","))
    }

    fn zip(entries: &[(String, Vec<u8>)]) -> Vec<u8> {
        let mut cursor = Cursor::new(Vec::new());
        let mut archive = ZipWriter::new(&mut cursor);
        for (name, bytes) in entries {
            archive.start_file(name, SimpleFileOptions::default()).unwrap();
            archive.write_all(bytes).unwrap();
        }
        archive.finish().unwrap();
        cursor.into_inner()
    }

    fn pack(frame: Vec<u8>) -> Vec<u8> {
        let mut entries: Vec<(String, Vec<u8>)> = vec![("manifest.json".to_owned(), manifest().into_bytes())];
        entries.extend(STATES.iter().map(|state| (format!("assets/{state}-00.png"), frame.clone())));
        zip(&entries)
    }

    #[test]
    fn imports_complete_png_and_webp_frames() {
        let root = std::env::temp_dir().join(format!("yonder-pet-{}-{}", std::process::id(), SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos()));
        fs::create_dir_all(&root).unwrap();
        fs::write(root.join("pack.zip"), pack(png())).unwrap();
        import_pet_pack(&root.join("pack.zip"), &root.join("packs")).unwrap();
        assert!(root.join("packs/current/assets/idle-00.png").exists());

        fs::write(root.join("webp.zip"), pack(webp())).unwrap();
        import_pet_pack(&root.join("webp.zip"), &root.join("packs")).unwrap();
        let active = active_pack(&root.join("packs")).unwrap();
        assert_eq!(active.assets.len(), STATES.len());
        assert_eq!(active.manifest["states"]["idle"]["files"][0], "idle-00.png");
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn rejects_missing_state_and_keeps_current_pack() {
        let root = std::env::temp_dir().join(format!("yonder-pet-fail-{}-{}", std::process::id(), SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos()));
        fs::create_dir_all(&root.join("packs/current/assets")).unwrap();
        fs::write(root.join("packs/current/assets/idle-00.png"), b"old").unwrap();
        fs::write(root.join("bad.zip"), pack(png())).unwrap();
        let invalid = manifest().replacen("\"listening\":", "\"missing\":", 1);
        let mut invalid_entries: Vec<(String, Vec<u8>)> = vec![("manifest.json".to_owned(), invalid.into_bytes())];
        invalid_entries.extend(STATES.iter().map(|state| (format!("assets/{state}-00.png"), png())));
        fs::write(root.join("invalid.zip"), zip(&invalid_entries)).unwrap();
        assert_eq!(import_pet_pack(&root.join("invalid.zip"), &root.join("packs")), Err(PetPackError::InvalidManifest));
        assert_eq!(fs::read(root.join("packs/current/assets/idle-00.png")).unwrap(), b"old");
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn rejects_path_escape_and_corrupt_image() {
        let escape = [("manifest.json".to_owned(), manifest().into_bytes()), ("../escape.png".to_owned(), png())];
        assert_eq!(validate(&zip(&escape)), Err(PetPackError::InvalidManifest));
        let mut corrupt: Vec<(String, Vec<u8>)> = vec![("manifest.json".to_owned(), manifest().into_bytes())];
        corrupt.extend(STATES.iter().map(|state| (format!("assets/{state}-00.png"), b"broken".to_vec())));
        assert_eq!(validate(&zip(&corrupt)), Err(PetPackError::InvalidImage));
    }
}
