use serde::Deserialize;
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

#[derive(serde::Serialize)]
struct Entry { path: String, name: String, directory: bool }
#[derive(Deserialize)]
struct Rename { from: String, to_name: String }
#[derive(serde::Serialize)]
struct BatchResult { count: usize, remaining: usize, entries: Vec<Entry>, previous: Vec<String> }
struct History(Mutex<Vec<Vec<(PathBuf, PathBuf)>>>);

#[tauri::command]
fn list_folder(folder: String) -> Result<Vec<Entry>, String> {
    let mut entries = Vec::new();
    let mut pending = vec![PathBuf::from(folder)];
    while let Some(directory) = pending.pop() {
        for result in fs::read_dir(&directory).map_err(|e| e.to_string())? {
            let e = result.map_err(|e| e.to_string())?;
            let file_type = e.file_type().map_err(|e| e.to_string())?;
            if file_type.is_dir() { pending.push(e.path()); }
            else if file_type.is_file() {
                entries.push(Entry { path: e.path().to_string_lossy().into_owned(), name: e.file_name().to_string_lossy().into_owned(), directory: false });
            }
        }
    }
    entries.sort_by(|a,b| a.name.cmp(&b.name));
    Ok(entries)
}
fn to_entry(path: &Path) -> Entry {
    Entry { path: path.to_string_lossy().into_owned(), name: path.file_name().unwrap_or_default().to_string_lossy().into_owned(), directory: path.is_dir() }
}
fn execute(operations: &[(PathBuf,PathBuf)]) -> Result<(),String> {
    let mut sources = HashSet::new();
    let mut targets = HashSet::new();
    for (from,to) in operations {
        if !from.exists() { return Err(format!("Source missing: {}",from.display())); }
        if !sources.insert(from.clone()) || !targets.insert(to.clone()) { return Err("Duplicate source or target".into()); }
        if to.exists() { return Err(format!("Destination already exists: {}",to.display())); }
        if operations.iter().any(|(other,_)| from != other && other.starts_with(from)) { return Err("Cannot rename a folder and its descendants together".into()); }
    }
    let mut completed: Vec<(PathBuf,PathBuf)> = Vec::new();
    for (from,to) in operations {
        if let Err(e) = fs::rename(from,to) {
            let mut rollback_errors = Vec::new();
            for (a,b) in completed.iter().rev() { if let Err(error) = fs::rename(b,a) { rollback_errors.push(error.to_string()); } }
            return Err(format!("Rename failed: {e}. Rollback errors: {}",rollback_errors.join("; ")));
        }
        completed.push((from.clone(),to.clone()));
    }
    Ok(())
}
#[tauri::command]
fn apply_renames(changes: Vec<Rename>, history: tauri::State<History>) -> Result<BatchResult,String> {
    let mut operations = Vec::new();
    for change in changes {
        if change.to_name.is_empty() || change.to_name == "." || change.to_name == ".." || change.to_name.contains('/') || change.to_name.contains(char::from(92)) || change.to_name.contains(char::from(0)) { return Err("Invalid target name".into()); }
        let from = PathBuf::from(change.from);
        let dest = from.parent().unwrap_or(Path::new("")).join(change.to_name);
        if dest != from { operations.push((from,dest)); }
    }
    if operations.is_empty() { return Err("Nothing to rename".into()); }
    let mut journal = history.0.lock().map_err(|e|e.to_string())?;
    execute(&operations)?;
    let entries = operations.iter().map(|(_,to)|to_entry(to)).collect();
    let previous = operations.iter().map(|(from,_)|from.to_string_lossy().into_owned()).collect();
    journal.push(operations);
    if journal.len() > 100 { journal.remove(0); }
    Ok(BatchResult { count: journal.last().unwrap().len(), remaining: journal.len(), entries, previous })
}
#[tauri::command]
fn undo_renames(history: tauri::State<History>) -> Result<BatchResult,String> {
    let mut journal = history.0.lock().map_err(|e|e.to_string())?;
    let batch = journal.last().ok_or("Nothing to undo")?;
    let reverse: Vec<(PathBuf,PathBuf)> = batch.iter().rev().map(|(a,b)|(b.clone(),a.clone())).collect();
    execute(&reverse)?;
    let entries = reverse.iter().map(|(_,to)|to_entry(to)).collect();
    let previous = reverse.iter().map(|(from,_)|from.to_string_lossy().into_owned()).collect();
    let count = reverse.len();
    journal.pop();
    Ok(BatchResult { count, remaining: journal.len(), entries })
}
#[tauri::command]
fn undo_count(history: tauri::State<History>) -> Result<usize,String> {
    Ok(history.0.lock().map_err(|e|e.to_string())?.len())
}
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
      .manage(History(Mutex::new(Vec::new())))
      .plugin(tauri_plugin_dialog::init())
      .invoke_handler(tauri::generate_handler![list_folder,apply_renames,undo_renames,undo_count])
      .run(tauri::generate_context!())
      .expect("error while running Enumu");
}
