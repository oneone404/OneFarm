use windows::Win32::UI::WindowsAndMessaging::*;
use windows::Win32::UI::HiDpi::*;
use windows::Win32::Foundation::*;
use windows::Win32::Graphics::Gdi::*;
use image::{GenericImageView, RgbaImage, ImageBuffer};
use std::fs;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use tauri::State;
use serde::{Serialize, Deserialize};
use std::process::Command;
use std::os::windows::process::CommandExt;

mod capture;
mod recognize_fast;

use capture::WgcGrabber;
use recognize_fast::FastRecognizer;

const BASE_W: u32 = 960;
const BASE_H: u32 = 540;
const CREATE_NO_WINDOW: u32 = 0x08000000;

#[derive(Serialize, Deserialize, Clone)]
struct DeviceInfo {
    index: i32,
    serial: String,
    title: String,
    handle: isize,
    bind_handle: isize,
}

struct CachedTemplate {
    dimensions: (u32, u32),
    data: Arc<Vec<u8>>,
}

struct AppState {
    grabbers: Mutex<HashMap<isize, WgcGrabber>>,
    active_device: Mutex<Option<DeviceInfo>>,
    template_cache: Mutex<HashMap<String, CachedTemplate>>,
}

#[tauri::command]
fn get_templates(state: State<'_, AppState>) -> Vec<String> {
    let mut names = Vec::new();
    let mut cache = state.template_cache.lock().unwrap();
    cache.clear(); // Làm mới Cache khi quét lại

    if let Ok(entries) = fs::read_dir("templates") {
        for entry in entries.flatten() {
            let path = entry.path();
            if let Some(ext) = path.extension() {
                if ext == "png" {
                    let name = path.file_name().unwrap().to_string_lossy().into_owned();
                    names.push(name.clone());
                    
                    // Nạp luôn vào RAM
                    if let Ok(img) = image::open(&path) {
                        let (w, h) = img.dimensions();
                        let data = Arc::new(img.to_rgb8().into_raw());
                        cache.insert(name, CachedTemplate { dimensions: (w, h), data });
                    }
                }
            }
        }
    }
    println!("Da nap {} mau vao RAM Cache.", names.len());
    names
}

fn get_ld_path() -> String {
    "C:\\LDPlayer\\LDPlayer9\\ldconsole.exe".to_string()
}

#[tauri::command]
fn get_devices() -> std::result::Result<Vec<DeviceInfo>, String> {
    let ld_path = get_ld_path();
    let output = Command::new(&ld_path)
        .arg("list2")
        .creation_flags(CREATE_NO_WINDOW)
        .output()
        .map_err(|_| format!("Không thấy ldconsole.exe tại {}", ld_path))?;
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut devices = Vec::new();
    
    for line in stdout.lines() {
        let parts: Vec<&str> = line.split(',').collect();
        if parts.len() >= 7 {
            let index = parts[0].parse::<i32>().unwrap_or(-1);
            let title = parts[1].to_string();
            let handle = parts[2].parse::<isize>().unwrap_or(0);
            let bind_handle = parts[3].parse::<isize>().unwrap_or(0);
            let is_in_android = parts[4] == "1";
            
            if handle != 0 && is_in_android {
                let adb_port = 5555 + (index * 2);
                devices.push(DeviceInfo {
                    index,
                    serial: format!("127.0.0.1:{}", adb_port),
                    title,
                    handle,
                    bind_handle,
                });
            }
        }
    }
    Ok(devices)
}

#[tauri::command]
fn set_active_device(state: State<'_, AppState>, device: DeviceInfo) -> std::result::Result<String, String> {
    let mut active = state.active_device.lock().unwrap();
    *active = Some(device.clone());
    Ok(format!("🎯 Kết nối: {} | Port: {}", device.title, device.serial))
}

#[tauri::command]
fn resize_ld(state: State<'_, AppState>) -> std::result::Result<String, String> {
    let device = state.active_device.lock().unwrap().clone().ok_or("Chưa chọn thiết bị!")?;
    
    // Xóa grabber cũ của thiết bị này để force cập nhật lại resolution nếu cần
    state.grabbers.lock().unwrap().remove(&device.handle);
    
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

fn get_or_create_grabber(state: &State<'_, AppState>) -> std::result::Result<(WgcGrabber, HWND, HWND, String), String> {
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

#[tauri::command]
fn capture_screen(state: State<'_, AppState>) -> std::result::Result<String, String> {
    let (g, parent_hwnd, bind_hwnd, _) = get_or_create_grabber(&state)?;
    unsafe {
        let mut c_rect = RECT::default();
        let _ = GetClientRect(bind_hwnd, &mut c_rect);
        let aw = c_rect.right as u32; let ah = c_rect.bottom as u32;
        let frame = g.capture_frame().map_err(|e| format!("{:?}", e))?;
        let (cw, ch) = g.get_resolution();
        
        let mut pt = POINT::default(); let mut rect = RECT::default();
        let _ = ClientToScreen(bind_hwnd, &mut pt);
        let _ = GetWindowRect(parent_hwnd, &mut rect);
        let ox = (pt.x - rect.left).max(0) as u32;
        let oy = (pt.y - rect.top).max(0) as u32;

        let mut screen_img: RgbaImage = ImageBuffer::new(aw, ah);
        let screen_mut: &mut [u8] = &mut *screen_img;
        let cw_bytes = cw as usize * 4;
        let aw_bytes = aw as usize * 4;

        for y in 0..ah as usize {
            let sy = y + oy as usize;
            if sy >= ch as usize { break; }
            let src_row_start = sy * cw_bytes + ox as usize * 4;
            let dest_row_start = y * aw_bytes;
            let copy_width = (cw as usize).saturating_sub(ox as usize).min(aw as usize);
            let src_ptr = frame.as_ptr().add(src_row_start);
            let dest_ptr = screen_mut.as_mut_ptr().add(dest_row_start);

            for i in 0..copy_width {
                let s_idx = i * 4;
                let d_idx = i * 4;
                let b = *src_ptr.add(s_idx);
                let g = *src_ptr.add(s_idx + 1);
                let r = *src_ptr.add(s_idx + 2);
                *dest_ptr.add(d_idx) = r;
                *dest_ptr.add(d_idx + 1) = g;
                *dest_ptr.add(d_idx + 2) = b;
                *dest_ptr.add(d_idx + 3) = 255;
            }
        }

        let diff_w = (aw as i32 - BASE_W as i32).abs();
        let diff_h = (ah as i32 - BASE_H as i32).abs();
        let norm = if diff_w > 4 || diff_h > 4 {
            image::imageops::resize(&screen_img, BASE_W, BASE_H, image::imageops::FilterType::Nearest)
        } else { screen_img };
        norm.save("debug_view.png").map_err(|e| e.to_string())?;
        Ok("Da luu anh vao debug_view.png".to_string())
    }
}

#[tauri::command]
async fn test_template(state: State<'_, AppState>, name: String) -> std::result::Result<String, String> {
    let start_total = std::time::Instant::now();
    
    // Logic Cache Template
    let (template_data, tw, th) = {
        let mut cache = state.template_cache.lock().unwrap();
        if let Some(c) = cache.get(&name) {
            (Arc::clone(&c.data), c.dimensions.0, c.dimensions.1)
        } else {
            let path = format!("templates/{}", name);
            let img = image::open(&path).map_err(|e| e.to_string())?;
            let (w, h) = img.dimensions();
            let data = Arc::new(img.to_rgb8().into_raw());
            cache.insert(name.clone(), CachedTemplate { dimensions: (w, h), data: Arc::clone(&data) });
            (data, w, h)
        }
    };

    let (g, _, bind_hwnd, serial) = get_or_create_grabber(&state)?;
    unsafe {
        let mut c_rect = RECT::default();
        let _ = GetClientRect(bind_hwnd, &mut c_rect);
        let aw = c_rect.right as u32; let ah = c_rect.bottom as u32;
        let frame = g.capture_frame().map_err(|e| format!("{:?}", e))?;
        let (cw, ch) = g.get_resolution();
        
        let mut pt = POINT::default(); let mut rect = RECT::default();
        let _ = ClientToScreen(bind_hwnd, &mut pt);
        let _ = GetWindowRect(HWND(state.active_device.lock().unwrap().as_ref().unwrap().handle as *mut _), &mut rect);
        let ox = (pt.x - rect.left).max(0) as u32;
        let oy = (pt.y - rect.top).max(0) as u32;

        let mut screen_img: RgbaImage = ImageBuffer::new(aw, ah);
        let screen_mut: &mut [u8] = &mut *screen_img;
        let cw_bytes = cw as usize * 4;
        let aw_bytes = aw as usize * 4;

        for y in 0..ah as usize {
            let sy = y + oy as usize;
            if sy >= ch as usize { break; }
            let src_row_start = sy * cw_bytes + ox as usize * 4;
            let dest_row_start = y * aw_bytes;
            let copy_width = (cw as usize).saturating_sub(ox as usize).min(aw as usize);
            let src_ptr = frame.as_ptr().add(src_row_start);
            let dest_ptr = screen_mut.as_mut_ptr().add(dest_row_start);

            for i in 0..copy_width {
                let s_idx = i * 4;
                let d_idx = i * 4;
                let b = *src_ptr.add(s_idx);
                let g = *src_ptr.add(s_idx + 1);
                let r = *src_ptr.add(s_idx + 2);
                *dest_ptr.add(d_idx) = r;
                *dest_ptr.add(d_idx + 1) = g;
                *dest_ptr.add(d_idx + 2) = b;
                *dest_ptr.add(d_idx + 3) = 255;
            }
        }

        let mut logs = Vec::new();
        let diff_w = (aw as i32 - BASE_W as i32).abs();
        let diff_h = (ah as i32 - BASE_H as i32).abs();
        
        let norm = if diff_w > 4 || diff_h > 4 {
            logs.push(format!("Chuan hoa {}x{} -> {}x{} (Nearest)", aw, ah, BASE_W, BASE_H));
            image::imageops::resize(&screen_img, BASE_W, BASE_H, image::imageops::FilterType::Nearest)
        } else {
            logs.push(format!("Bo qua Chuan hoa do sai lech nho ({}x{})", aw, ah));
            screen_img
        };
        
        let norm_w = norm.width();
        let norm_h = norm.height();
        let screen_rgba = norm.into_raw();

        let start_recog = std::time::Instant::now();
        if let Some((fx, fy, score)) = FastRecognizer::find_template_step(&screen_rgba, norm_w as usize, norm_h as usize, 4, &template_data, tw as usize, th as usize, 25) {
            let recog_time = start_recog.elapsed().as_millis();
            let tx = (fx as f64 + tw as f64 / 2.0) as i32;
            let ty = (fy as f64 + th as f64 / 2.0) as i32;
            
            let actual_tx = (tx as f64 * aw as f64 / norm_w as f64) as i32;
            let actual_ty = (ty as f64 * ah as f64 / norm_h as f64) as i32;
            
            let _ = PostMessageW(Some(bind_hwnd), WM_LBUTTONDOWN, WPARAM(1), LPARAM(((actual_ty << 16) | (actual_tx & 0xFFFF)) as isize));
            let _ = PostMessageW(Some(bind_hwnd), WM_LBUTTONUP, WPARAM(0), LPARAM(((actual_ty << 16) | (actual_tx & 0xFFFF)) as isize));
            
            let total_time = start_total.elapsed().as_millis();
            logs.push(format!("[KHOP] ({}, {}) | Score: {}", fx, fy, score));
            logs.push(format!("[WIN32] Click {} tai ({}, {}) (scaled tu {}, {})", serial, actual_tx, actual_ty, tx, ty));
            logs.push(format!("[TIME] Quet: {}ms | Tong: {}ms", recog_time, total_time));
            Ok(logs.join("\n"))
        } else {
            let mut err_msg = format!("Khong tim thay mau! ({}ms)", start_total.elapsed().as_millis());
            if !logs.is_empty() {
                err_msg = format!("{}\n{}", logs.join("\n"), err_msg);
            }
            Err(err_msg)
        }
    }
}

#[tauri::command]
fn check_session(state: tauri::State<'_, AppState>, handle: isize) -> bool {
    let grabbers = state.grabbers.lock().unwrap();
    grabbers.contains_key(&handle)
}

#[tauri::command]
fn connect_session(state: State<'_, AppState>, device: DeviceInfo) -> std::result::Result<String, String> {
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
fn disconnect_session(state: State<'_, AppState>, handle: isize) {
    let mut grabbers = state.grabbers.lock().unwrap();
    if let Some(g) = grabbers.get(&handle) {
        g.close();
    }
    grabbers.remove(&handle);
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
        })
        .invoke_handler(tauri::generate_handler![get_templates, test_template, resize_ld, capture_screen, get_devices, set_active_device, check_session, connect_session, disconnect_session])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
