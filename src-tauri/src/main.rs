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

fn main() {
    unsafe {
        let _ = SetProcessDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2);
    }
    tauri::Builder::default()
        .manage(AppState { 
            grabbers: Mutex::new(HashMap::new()), 
            active_device: Mutex::new(None),
            template_cache: Mutex::new(HashMap::new()),
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_templates,
            commands::test_template,
            commands::resize_ld,
            commands::capture_screen,
            commands::get_devices,
            commands::set_active_device,
            commands::check_session,
            commands::connect_session,
            commands::disconnect_session
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
