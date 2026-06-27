#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::fs::{self, File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::Manager;

mod api;
mod errors;
mod instalock;

struct AppState {
    is_running: std::sync::Arc<Mutex<bool>>,
}

impl Clone for AppState {
    fn clone(&self) -> Self {
        Self {
            is_running: self.is_running.clone(),
        }
    }
}

fn get_data_dir() -> PathBuf {
    let mut dir: PathBuf = tauri::api::path::data_dir().unwrap();
    dir.push("VaLock");
    if !dir.exists() {
        let _ = fs::create_dir_all(&dir);
    }
    dir
}

fn get_active_profile_dir() -> PathBuf {
    let mut dir: PathBuf = get_data_dir();
    dir.push("active");
    dir
}

fn get_profiles_dir() -> PathBuf {
    let mut dir: PathBuf = get_data_dir();
    dir.push("profiles");
    dir
}

#[tauri::command]
fn get_active_profile() -> String {
    let dir: PathBuf = get_active_profile_dir();
    if dir.exists() {
        return fs::read_to_string(dir).expect("default");
    }
    "default".to_string()
}

fn get_config_dir() -> PathBuf {
    let mut dir: PathBuf = get_profiles_dir();
    dir.push(get_active_profile());
    dir
}

#[tauri::command]
fn get_agents() -> Result<Vec<api::Agent>, errors::MyErr> {
    Ok(api::get_agents()?)
}

#[tauri::command]
fn get_maps() -> Result<Vec<api::Map>, errors::MyErr> {
    Ok(api::get_maps()?)
}

#[tauri::command]
fn get_config() -> Result<Value, errors::MyErr> {
    let mut file = File::open(get_config_dir())?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    let data: Value = serde_json::from_str(&contents)?;
    Ok(data)
}

#[tauri::command]
fn set_agent_for_all_maps(name: String) -> Result<(), errors::MyErr> {
    let maps = api::get_maps()?;
    let mut data = json!({});
    for map in maps {
        data[map.name] = json!(name);
    }
    let mut file = File::create(get_config_dir())?;
    file.write_all(data.to_string().as_bytes())?;
    Ok(())
}

#[tauri::command]
fn set_agent_for_map(agent: String, map: String) -> Result<(), errors::MyErr> {
    let mut config = get_config()?;
    config[map] = Value::String(agent);
    let json_string = serde_json::to_string(&config)?;
    let mut file = OpenOptions::new()
        .write(true)
        .truncate(true)
        .open(get_config_dir())?;
    file.seek(SeekFrom::Start(0))?;
    file.write_all(json_string.as_bytes())?;
    file.sync_all()?;
    Ok(())
}

#[derive(Debug, Serialize, Deserialize)]
struct Profile {
    name: String,
    full_portrait: String,
}

#[tauri::command]
fn add_profile(name: String) -> Result<(), errors::MyErr> {
    let mut dir = get_profiles_dir();
    dir.push(name);
    if dir.exists() {
        return Err(errors::MyErr::DuplicateErr());
    }
    let mut file = File::create(dir)?;
    file.write_all(json!({}).to_string().as_bytes())?;
    Ok(())
}

#[tauri::command]
fn delete_profile(name: String) -> Result<(), errors::MyErr> {
    let mut dir = get_profiles_dir();
    dir.push(&name);
    fs::remove_file(dir)?;
    Ok(())
}

#[tauri::command]
fn get_profiles() -> Result<Vec<String>, errors::MyErr> {
    let dir = get_profiles_dir();
    let entries = fs::read_dir(dir)?;
    let profiles: Vec<String> = entries
        .filter_map(|entry| {
            if let Ok(entry) = entry {
                entry.file_name().to_str().map(String::from)
            } else {
                None
            }
        })
        .collect();
    Ok(profiles)
}

#[tauri::command]
fn set_active_profile(name: String) -> Result<(), errors::MyErr> {
    let dir = get_active_profile_dir();
    let mut data = File::create(dir)?;
    data.write_all(name.as_bytes())?;
    Ok(())
}

#[tauri::command]
async fn start_instalock(
    app: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
) -> Result<(), errors::MyErr> {
    {
        let running = state.is_running.lock().unwrap();
        if *running {
            return Err(errors::MyErr::CustomError("Insta-lock is already running".to_string()));
        }
    }

    let config_dir = get_config_dir();
    let mut file = File::open(&config_dir)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    let config_value: Value = serde_json::from_str(&contents)?;

    let mut config_map = std::collections::HashMap::new();
    if let Some(obj) = config_value.as_object() {
        for (key, val) in obj {
            if let Some(name) = val.as_str() {
                config_map.insert(key.clone(), name.to_string());
            }
        }
    }

    {
        let mut running = state.is_running.lock().unwrap();
        *running = true;
    }

    let app_handle = app.clone();
    let is_running = state.is_running.clone();

    std::thread::spawn(move || {
        let result = instalock::InstaLock::new(config_map);

        match result {
            Ok(locker) => {
                let _ = locker.run(move |status| {
                    let _ = app_handle.emit_all("instalock-status", status);
                });
            }
            Err(e) => {
                let _ = app.emit_all("instalock-status", format!("Error: {}", e));
            }
        }

        let mut running = is_running.lock().unwrap();
        *running = false;
        let _ = app.emit_all("instalock-stopped", ());
    });

    Ok(())
}

#[tauri::command]
fn stop_instalock(state: tauri::State<'_, AppState>) -> Result<(), errors::MyErr> {
    let mut running = state.is_running.lock().unwrap();
    *running = false;
    Ok(())
}

#[tauri::command]
fn is_instalock_running(state: tauri::State<'_, AppState>) -> bool {
    let running = state.is_running.lock().unwrap();
    *running
}

fn main() {
    if !get_config_dir().exists() {
        let _ = fs::create_dir_all(get_profiles_dir());
        if let Ok(mut file) = File::create(get_config_dir()) {
            let _ = file.write_all(json!({}).to_string().as_bytes());
        }
    }

    tauri::Builder::default()
        .manage(AppState {
            is_running: std::sync::Arc::new(Mutex::new(false)),
        })
        .invoke_handler(tauri::generate_handler![
            get_agents,
            get_maps,
            get_config,
            set_agent_for_all_maps,
            set_agent_for_map,
            add_profile,
            set_active_profile,
            get_profiles,
            get_active_profile,
            delete_profile,
            start_instalock,
            stop_instalock,
            is_instalock_running,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
