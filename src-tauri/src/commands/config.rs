use crate::config::persisted::AppConfig;
use crate::config::state::AppState;
use tauri::State;
use std::collections::HashMap;

#[tauri::command]
pub fn get_config() -> std::result::Result<AppConfig, String> {
    Ok(AppConfig::load())
}

#[tauri::command]
pub fn save_config(config: AppConfig) -> std::result::Result<String, String> {
    config.save()?;
    Ok("Da luu cau hinh.".to_string())
}

#[tauri::command]
pub fn get_purchase_history(state: State<'_, AppState>) -> std::result::Result<HashMap<String, u32>, String> {
    let history = state.seed_purchase_history.lock().unwrap();
    Ok(history.clone())
}

#[tauri::command]
pub fn clear_purchase_history(state: State<'_, AppState>) -> std::result::Result<(), String> {
    let mut history = state.seed_purchase_history.lock().unwrap();
    history.clear();
    Ok(())
}
