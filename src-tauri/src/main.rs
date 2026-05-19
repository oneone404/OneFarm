#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use std::sync::Mutex;
use std::collections::HashMap;
use windows::Win32::UI::HiDpi::{SetProcessDpiAwarenessContext, DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2};

mod core;
mod emulator;
mod config;
mod commands;
mod automation;

use config::state::AppState;

#[tauri::command]
fn restart_app(app_handle: tauri::AppHandle) {
    app_handle.restart();
}

fn main() {
    unsafe {
        let _ = SetProcessDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2);
    }
    tauri::Builder::default()
        .manage(AppState { 
            grabbers: Mutex::new(HashMap::new()), 
            active_device: Mutex::new(None),
            template_cache: Mutex::new(HashMap::new()),
            cancelled_devices: Mutex::new(std::collections::HashSet::new()),
            seed_purchase_history: Mutex::new(HashMap::new()),
        })
        .invoke_handler(tauri::generate_handler![
            commands::template::get_templates,
            commands::template::test_template,
            commands::device::resize_ld,
            commands::template::capture_screen,
            commands::device::get_devices,
            commands::device::set_active_device,
            commands::session::check_session,
            commands::session::connect_session,
            commands::session::disconnect_session,
            commands::template::test_all_templates,
            commands::template::check_seeds_templates,
            commands::template::test_digit_recognition,
            commands::device::cancel_device_actions,
            commands::script::run_buy_seeds_script,
            commands::script::run_buy_tools_script,
            commands::script::run_harvest_sell_script,
            commands::config::get_config,
            commands::config::save_config,
            commands::config::get_purchase_history,
            commands::config::clear_purchase_history,
            commands::template::get_seed_names,
            commands::template::get_tool_names,
            restart_app
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
