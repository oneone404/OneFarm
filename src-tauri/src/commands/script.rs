use tauri::State;
use crate::config::state::AppState;

#[tauri::command]
pub async fn run_buy_seeds_script(state: State<'_, AppState>, target_seeds: Vec<String>) -> std::result::Result<String, String> {
    crate::automation::buy_seeds::run_buy_seeds_script_logic(&state, target_seeds)
}

#[tauri::command]
pub async fn run_buy_tools_script(state: State<'_, AppState>, target_tools: Vec<String>) -> std::result::Result<String, String> {
    crate::automation::buy_tools::run_buy_tools_script_logic(&state, target_tools)
}

#[tauri::command]
pub async fn run_harvest_sell_script(state: State<'_, AppState>) -> std::result::Result<String, String> {
    crate::automation::harvest_sell::run_harvest_sell_script_logic(&state)
}
