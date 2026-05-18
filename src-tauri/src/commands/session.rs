use tauri::State;
use windows::Win32::Foundation::HWND;
use crate::config::state::{AppState, DeviceInfo};
use crate::core::capture::WgcGrabber;

#[tauri::command]
pub fn check_session(state: State<'_, AppState>, handle: isize) -> bool {
    let grabbers = state.grabbers.lock().unwrap();
    grabbers.contains_key(&handle)
}

#[tauri::command]
pub fn connect_session(state: State<'_, AppState>, device: DeviceInfo) -> std::result::Result<String, String> {
    let mut grabbers = state.grabbers.lock().unwrap();
    let hwnd = HWND(device.handle as *mut _);
    if grabbers.contains_key(&device.handle) {
        return Ok("Connected".to_string());
    }
    let g = WgcGrabber::new(hwnd).map_err(|e| format!("{:?}", e))?;
    grabbers.insert(device.handle, g.clone_instance());
    Ok("Connected".to_string())
}

#[tauri::command]
pub fn disconnect_session(state: State<'_, AppState>, handle: isize) {
    let mut grabbers = state.grabbers.lock().unwrap();
    if let Some(g) = grabbers.get(&handle) {
        g.close();
    }
    grabbers.remove(&handle);
}
