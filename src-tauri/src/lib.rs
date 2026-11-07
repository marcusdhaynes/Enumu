use serde::Deserialize;
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(serde::Serialize)]
struct Entry { path: String, name: String, directory: bool }
#[derive(Deserialize)]
struct Rename { from: String, to_name: String }

#[tauri::command]
fn list_folder(folder: String) -> Result<Vec<Entry>, String> {
    let mut entries = Vec::new();
    let iter = fs::read_dir(&folder).map_err(|e| e.to_string())?;
    for result in iter {
        let e = result.map_err(|e| e.to_string())?;
        let file_type = e.file_type().map_err(|e| e.to_string())?;
        entries.push(Entry { path: e.path().to_string_lossy().into_owned(), name: e.file_name().to_string_lossy().into_owned(), directory: file_type.is_dir() });
    }
    entries.sort_by(|a,b| a.name.cmp(&b.name));
    Ok(entries)
}

#[tauri::command]
fn apply_renames(changes: Vec<Rename>) -> Result<String, String> {
    let mut targets = HashSet::<PathBuf>::new();
    let mut sources = HashSet::<PathBuf>::new();
    let mut operations = Vec::<(PathBuf, PathBuf)>::new();
    for change in changes {
        let from = PathBuf::from(&change.from);
        if !from.exists() { return Err(format!("Missing source: {}", from.display())); }
        if change.to_name.is_empty() || change.to_name == "." || change.to_name == ".." || change.to_name.contains('/') || change.to_name.contains('\\\\') || change.to_name.contains('\\0') { return Err("Invalid target name".into()); }
        let dest = from.parent().unwrap_or(Path::new("")).join(&change.to_name);
        if dest == from { continue; }
        if !sources.insert(from.clone()) || !targets.insert(dest.clone()) { return Err("Duplicate source or target".into()); }
        operations.push((from, dest));
    }
    for (from, dest) in &operations {
        if dest.exists() { return Err(format!("Destination already exists: {}", dest.display())); }
        if operations.iter().any(|(other, _)| from != other && other.starts_with(from)) { return Err("Cannot rename a folder alongside its descendants in the same batch".into()); }
    }
    let mut completed = Vec::new();
    for (from, dest) in &operations {
        match fs::rename(from, dest) {
            Ok(()) => completed.push((from.clone(), dest.clone())),
            Err(e) => {
                for (original, renamed) in completed.iter().rev() { let _ = fs::rename(renamed, original); }
                return Err(format!("Rename failed: {}. Previous renames were rolled back where possible.", e));
            }
        }
    }
    Ok(format!("Renamed {} items successfully", completed.len()))
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
      .plugin(tauri_plugin_dialog::init())
      .invoke_handler(tauri::generate_handler![list_folder, apply_renames])
      .run(tauri::generate_context!())
      .expect("error while running Enumu");
}
