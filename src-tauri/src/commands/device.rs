use tauri::State;
use windows::Win32::Foundation::{HWND, RECT};
use windows::Win32::UI::WindowsAndMessaging::{GetClientRect, GetWindowRect, SetWindowPos, SWP_NOZORDER, SWP_NOMOVE, SWP_FRAMECHANGED};
use crate::config::state::{AppState, DeviceInfo};
use crate::core::capture::WgcGrabber;
use crate::emulator::ldplayer::get_ld_devices;

#[tauri::command]
pub fn get_devices() -> std::result::Result<Vec<DeviceInfo>, String> {
    get_ld_devices()
}

#[tauri::command]
pub fn set_active_device(state: State<'_, AppState>, device: DeviceInfo) -> std::result::Result<String, String> {
    let mut active = state.active_device.lock().unwrap();
    *active = Some(device.clone());
    Ok(format!("🎯 Kết nối: {} | Port: {}", device.title, device.serial))
}

#[tauri::command]
pub fn resize_ld(state: State<'_, AppState>) -> std::result::Result<String, String> {
    let device = state.active_device.lock().unwrap().clone().ok_or("Chưa chọn thiết bị!")?;
    
    // Đóng Grabber cũ một cách an toàn để giải phóng tài nguyên trước khi thay đổi kích thước
    {
        let mut grabbers = state.grabbers.lock().unwrap();
        if let Some(g) = grabbers.get(&device.handle) {
            g.close();
        }
        grabbers.remove(&device.handle);
    }
    
    unsafe {
        let hwnd = HWND(device.handle as *mut _);
        let render_h = HWND(device.bind_handle as *mut _);
        let mut p_rect = RECT::default();
        let _ = GetWindowRect(hwnd, &mut p_rect);
        let mut r_rect = RECT::default();
        let _ = GetWindowRect(render_h, &mut r_rect);
        
        let ew = (p_rect.right - p_rect.left) - (r_rect.right - r_rect.left);
        let eh = (p_rect.bottom - p_rect.top) - (r_rect.bottom - r_rect.top);
        let _ = SetWindowPos(hwnd, None, 0, 0, 960 + ew, 540 + eh, SWP_NOZORDER | SWP_NOMOVE | SWP_FRAMECHANGED);
        Ok(format!("Da chuan hoa {} ve 960x540.", device.title))
    }
}

#[tauri::command]
pub fn cancel_device_actions(state: State<'_, AppState>, handle: isize) -> std::result::Result<String, String> {
    let mut cancelled = state.cancelled_devices.lock().unwrap();
    cancelled.insert(handle);
    Ok(format!("Da gui yeu cau dung cho thiet bi: {}", handle))
}

pub fn get_or_create_grabber(state: &State<'_, AppState>) -> std::result::Result<(WgcGrabber, HWND, HWND, String), String> {
    let device = state.active_device.lock().unwrap().clone().ok_or("Chưa chọn thiết bị!")?;
    let mut grabbers = state.grabbers.lock().unwrap();
    
    let hwnd = HWND(device.handle as *mut _);
    let bind_hwnd = HWND(device.bind_handle as *mut _);

    if let Some(g) = grabbers.get(&device.handle) {
        let mut rect = RECT::default();
        unsafe {
            let _ = GetClientRect(hwnd, &mut rect);
        }
        let current_w = (rect.right - rect.left) as u32;
        let current_h = (rect.bottom - rect.top) as u32;
        let (gw, gh) = g.get_resolution();
        if current_w != 0 && current_h != 0 && (current_w != gw || current_h != gh) {
            println!("Phat hien cua so doi kich thuoc: {}x{} -> {}x{}. Dang tao lai Grabber...", gw, gh, current_w, current_h);
            g.close();
            grabbers.remove(&device.handle);
        } else {
            return Ok((g.clone_instance(), hwnd, bind_hwnd, device.serial));
        }
    }
    
    let g = WgcGrabber::new(hwnd).map_err(|e| format!("{:?}", e))?;
    grabbers.insert(device.handle, g.clone_instance());
    Ok((g, hwnd, bind_hwnd, device.serial))
}
