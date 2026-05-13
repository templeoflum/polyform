use std::path::PathBuf;

use polyform_core::factory_presets::factory_presets;
use polyform_core::preset_io::{
    PresetFile, PresetKind, export_preset_file_json, import_preset_file_json,
};

/// Entry in the preset library.
#[derive(Debug, Clone)]
pub struct PresetEntry {
    pub name: String,
    pub path: PathBuf,
    pub kind: PresetKind,
}

/// Return the root directory for the preset library.
pub fn library_root() -> Option<PathBuf> {
    dirs::data_dir().map(|d| d.join("polyform").join("presets"))
}

/// Ensure the library directory structure exists.
pub fn ensure_library_dirs() -> Option<PathBuf> {
    let root = library_root()?;
    let kinds = [
        PresetKind::Full,
        PresetKind::Operations,
        PresetKind::Lift,
        PresetKind::Camera,
        PresetKind::Phosphor,
        PresetKind::Sampling,
    ];
    for kind in &kinds {
        let dir = root.join(kind.dir_name());
        if let Err(e) = std::fs::create_dir_all(&dir) {
            eprintln!("Failed to create preset dir {}: {e}", dir.display());
            return None;
        }
    }
    Some(root)
}

/// List all presets of a given kind.
pub fn list_presets(kind: PresetKind) -> Vec<PresetEntry> {
    let root = match library_root() {
        Some(r) => r,
        None => return Vec::new(),
    };
    let dir = root.join(kind.dir_name());
    let mut entries = Vec::new();
    if let Ok(read_dir) = std::fs::read_dir(&dir) {
        for entry in read_dir.flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) == Some("json") {
                let name = path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("unknown")
                    .to_string();
                entries.push(PresetEntry { name, path, kind });
            }
        }
    }
    entries.sort_by(|a, b| a.name.cmp(&b.name));
    entries
}

/// Load a preset file from disk.
pub fn load_preset(path: &std::path::Path) -> Result<PresetFile, String> {
    let json = std::fs::read_to_string(path).map_err(|e| format!("Read error: {e}"))?;
    import_preset_file_json(&json).map_err(|e| format!("Parse error: {e}"))
}

/// Install or update factory presets in the library.
/// Overwrites existing factory presets to pick up fixes.
pub fn install_factory_presets() {
    let root = match ensure_library_dirs() {
        Some(r) => r,
        None => return,
    };
    let full_dir = root.join("full");

    let presets = factory_presets();
    for (name, preset) in presets {
        let file = PresetFile::Full(preset);
        let json = export_preset_file_json(&file, true);
        let path = full_dir.join(format!("{name}.json"));
        if let Err(e) = std::fs::write(&path, &json) {
            eprintln!("Failed to write factory preset \"{name}\": {e}");
        }
    }
}

/// Save a preset file to the library.
pub fn save_preset(name: &str, file: &PresetFile) -> Result<PathBuf, String> {
    let root = ensure_library_dirs().ok_or_else(|| "Cannot find data directory".to_string())?;
    let kind = file.kind();
    let dir = root.join(kind.dir_name());
    let path = dir.join(format!("{name}.json"));
    let json = export_preset_file_json(file, true);
    std::fs::write(&path, json).map_err(|e| format!("Write error: {e}"))?;
    Ok(path)
}
