use windows::Win32::UI::WindowsAndMessaging::*;
use windows::Win32::UI::HiDpi::*;
use windows::Win32::Foundation::*;
use windows::core::*;
use windows::Win32::Graphics::Gdi::*;
use image::{GenericImageView, RgbaImage, ImageBuffer, Rgba};
use std::fs;
use std::sync::Mutex;
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
    data: Vec<u8>,
}

struct AppState {
    grabber: Mutex<Option<WgcGrabber>>,
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
                        let data = img.to_rgb8().into_raw();
                        cache.insert(name, CachedTemplate { dimensions: (w, h), data });
                    }
                }
            }
        }
    }
    println!("🚀 Đã nạp {} mẫu vào RAM Cache.", names.len());
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
    let mut grabber = state.grabber.lock().unwrap();
    *active = Some(device.clone());
    *grabber = None; // Reset sẽ kích hoạt Drop() ở capture.rs giúp dọn dẹp GPU
    Ok(format!("🎯 Kết nối: {} | Port: {}", device.title, device.serial))
}

#[tauri::command]
fn resize_ld(state: State<'_, AppState>) -> std::result::Result<String, String> {
    let device = state.active_device.lock().unwrap().clone().ok_or("Chưa chọn thiết bị!")?;
    *state.grabber.lock().unwrap() = None;
    
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
        Ok(format!("✅ Đã chuẩn hóa {} về 960x540.", device.title))
    }
}

fn get_or_create_grabber(state: &State<'_, AppState>) -> std::result::Result<(WgcGrabber, HWND, HWND, String), String> {
    let device = state.active_device.lock().unwrap().clone().ok_or("Chưa chọn thiết bị!")?;
    let mut lock = state.grabber.lock().unwrap();
    let hwnd = HWND(device.handle as *mut _);
    let bind_hwnd = HWND(device.bind_handle as *mut _);

    if let Some(g) = &*lock {
        return Ok((g.clone_instance(), hwnd, bind_hwnd, device.serial));
    }
    
    let g = WgcGrabber::new(hwnd).map_err(|e| format!("{:?}", e))?;
    *lock = Some(g.clone_instance());
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
        for y in 0..ah {
            for x in 0..aw {
                let sx = x + ox; let sy = y + oy;
                if sx < cw && sy < ch {
                    let idx = (sy * cw + sx) as usize * 4;
                    if idx + 2 < frame.len() {
                        screen_img.put_pixel(x, y, Rgba([frame[idx+2], frame[idx+1], frame[idx], 255]));
                    }
                }
            }
        }
        let norm = if aw != BASE_W || ah != BASE_H {
            image::imageops::resize(&screen_img, BASE_W, BASE_H, image::imageops::FilterType::Triangle)
        } else { screen_img };
        norm.save("debug_view.png").map_err(|e| e.to_string())?;
        Ok("📸 Đã lưu ảnh vào debug_view.png".to_string())
    }
}

#[tauri::command]
async fn test_template(state: State<'_, AppState>, name: String) -> std::result::Result<String, String> {
    let start_total = std::time::Instant::now();
    
    // Logic Cache Template
    let (template_data, tw, th) = {
        let mut cache = state.template_cache.lock().unwrap();
        if let Some(c) = cache.get(&name) {
            (c.data.clone(), c.dimensions.0, c.dimensions.1)
        } else {
            let path = format!("templates/{}", name);
            let img = image::open(&path).map_err(|e| e.to_string())?;
            let (w, h) = img.dimensions();
            let data = img.to_rgb8().into_raw();
            cache.insert(name.clone(), CachedTemplate { dimensions: (w, h), data: data.clone() });
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
        for y in 0..ah {
            for x in 0..aw {
                let sx = x + ox; let sy = y + oy;
                if sx < cw && sy < ch {
                    let idx = (sy * cw + sx) as usize * 4;
                    if idx + 2 < frame.len() {
                        screen_img.put_pixel(x, y, Rgba([frame[idx+2], frame[idx+1], frame[idx], 255]));
                    }
                }
            }
        }

        let mut logs = Vec::new();
        let norm = if aw != BASE_W || ah != BASE_H {
            logs.push(format!("🔄 Chuẩn hóa {}x{} -> {}x{}", aw, ah, BASE_W, BASE_H));
            image::imageops::resize(&screen_img, BASE_W, BASE_H, image::imageops::FilterType::Triangle)
        } else { screen_img };
        let screen_rgb = image::DynamicImage::ImageRgba8(norm).to_rgb8().into_raw();

        let start_recog = std::time::Instant::now();
        if let Some((fx, fy, score)) = FastRecognizer::find_template_step(&screen_rgb, BASE_W as usize, BASE_H as usize, &template_data, tw as usize, th as usize, 25) {
            let recog_time = start_recog.elapsed().as_millis();
            let tx = (fx as f64 + tw as f64 / 2.0) as i32;
            let ty = (fy as f64 + th as f64 / 2.0) as i32;
            
            let _ = PostMessageW(Some(bind_hwnd), WM_LBUTTONDOWN, WPARAM(1), LPARAM(((ty << 16) | (tx & 0xFFFF)) as isize));
            let _ = PostMessageW(Some(bind_hwnd), WM_LBUTTONUP, WPARAM(0), LPARAM(((ty << 16) | (tx & 0xFFFF)) as isize));
            
            let total_time = start_total.elapsed().as_millis();
            logs.push(format!("🎯 [KHỚP] ({}, {}) | Score: {}", fx, fy, score));
            logs.push(format!("🖱️ [WIN32] Click {} tại ({}, {})", serial, tx, ty));
            logs.push(format!("⏱️ [TIME] Quét: {}ms | Tổng: {}ms", recog_time, total_time));
            Ok(logs.join("\n"))
        } else {
            Err(format!("❌ Không tìm thấy mẫu! ({}ms)", start_total.elapsed().as_millis()))
        }
    }
}

fn main() {
    unsafe {
        let _ = SetProcessDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2);
    }
    tauri::Builder::default()
        .manage(AppState { 
            grabber: Mutex::new(None), 
            active_device: Mutex::new(None),
            template_cache: Mutex::new(HashMap::new()),
        })
        .invoke_handler(tauri::generate_handler![get_templates, test_template, resize_ld, capture_screen, get_devices, set_active_device])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
