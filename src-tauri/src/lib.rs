use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use tauri::Manager;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct VolunteerEntry {
    id: String,
    place: String,
    date: String,
    hours: f32,
    notes: String,
}

fn get_data_path(app: &tauri::AppHandle) -> PathBuf {
    let app_dir = app.path().app_data_dir().expect("Failed to get app data dir");
    fs::create_dir_all(&app_dir).expect("Failed to create app data dir");
    app_dir.join("volunteer_log.json")
}

fn load_entries(app: &tauri::AppHandle) -> Vec<VolunteerEntry> {
    let path = get_data_path(app);
    if path.exists() {
        let data = fs::read_to_string(&path).unwrap_or_default();
        serde_json::from_str(&data).unwrap_or_default()
    } else {
        Vec::new()
    }
}

fn save_entries(app: &tauri::AppHandle, entries: &[VolunteerEntry]) {
    let path = get_data_path(app);
    let data = serde_json::to_string_pretty(entries).expect("Failed to serialize entries");
    fs::write(&path, data).expect("Failed to write entries");
}

#[tauri::command]
fn get_entries(app: tauri::AppHandle) -> Vec<VolunteerEntry> {
    load_entries(&app)
}

#[tauri::command]
fn add_entry(app: tauri::AppHandle, place: String, date: String, hours: f32, notes: String) -> Vec<VolunteerEntry> {
    let mut entries = load_entries(&app);
    let entry = VolunteerEntry {
        id: uuid_v4(),
        place,
        date,
        hours,
        notes,
    };
    entries.push(entry);
    save_entries(&app, &entries);
    entries
}

#[tauri::command]
fn delete_entry(app: tauri::AppHandle, id: String) -> Vec<VolunteerEntry> {
    let mut entries = load_entries(&app);
    entries.retain(|e| e.id != id);
    save_entries(&app, &entries);
    entries
}

#[tauri::command]
fn update_entry(app: tauri::AppHandle, id: String, place: String, date: String, hours: f32, notes: String) -> Vec<VolunteerEntry> {
    let mut entries = load_entries(&app);
    if let Some(entry) = entries.iter_mut().find(|e| e.id == id) {
        entry.place = place;
        entry.date = date;
        entry.hours = hours;
        entry.notes = notes;
    }
    save_entries(&app, &entries);
    entries
}

fn uuid_v4() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    format!("{:x}-{:x}", now, rand_u32())
}

fn rand_u32() -> u32 {
    use std::collections::hash_map::RandomState;
    use std::hash::{BuildHasher, Hasher};
    RandomState::new().build_hasher().finish() as u32
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![get_entries, add_entry, delete_entry, update_entry])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
