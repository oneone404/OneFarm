use windows::Win32::UI::WindowsAndMessaging::*;
use windows::Win32::UI::HiDpi::*;
use windows::Win32::Foundation::*;
use windows::core::*;
use windows::Win32::System::WinRT::*;
use windows::Win32::Graphics::Gdi::*;
use image::{GenericImageView, RgbaImage, ImageBuffer, Rgba};
use std::time::{Duration, Instant};
use std::thread::sleep;
use std::collections::HashMap;
use std::fs;
use serde::{Serialize, Deserialize};

mod capture;
mod adb;
mod recognize_fast;

use capture::WgcGrabber;
use adb::AdbClient;
use recognize_fast::FastRecognizer;

#[derive(Serialize, Deserialize, Clone)]
struct Pos { x: u32, y: u32 }

struct Template {
    name: String,
    data_rgb: Vec<u8>,
    w: u32,
    h: u32,
}

// Độ phân giải thiết kế (Chuẩn để cắt template)
const BASE_W: u32 = 948;
const BASE_H: u32 = 584;

fn get_game_render_window(parent_hwnd: HWND) -> HWND {
    let mut render_hwnd = parent_hwnd;
    unsafe {
        let class_name: Vec<u16> = "RenderWindow".encode_utf16().chain(std::iter::once(0)).collect();
        let found = FindWindowExW(Some(parent_hwnd), None, PCWSTR(class_name.as_ptr()), None);
        if let Ok(h) = found {
            if !h.is_invalid() {
                render_hwnd = h;
            } else {
                let sub_class: Vec<u16> = "SubWin".encode_utf16().chain(std::iter::once(0)).collect();
                let sub = FindWindowExW(Some(parent_hwnd), None, PCWSTR(sub_class.as_ptr()), None);
                if let Ok(sh) = sub {
                    if !sh.is_invalid() {
                        render_hwnd = sh;
                    }
                }
            }
        }
    }
    render_hwnd
}

fn main() {
    println!("--- ULTRA-FAST AUTO: NORMALIZED SCALING MODE ---");
    println!("🛡️ Base Resolution: {}x{}", BASE_W, BASE_H);

    unsafe {
        let _ = RoInitialize(RO_INIT_SINGLETHREADED);
        let _ = SetProcessDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2);
    }

    let mut grabber: Option<WgcGrabber> = None;
    let mut target_hwnd: Option<HWND> = None;
    let mut render_hwnd_cached: Option<HWND> = None;
    let mut last_actual_res = (0u32, 0u32);
    let mut positions: HashMap<String, Pos> = fs::read_to_string("positions.json")
        .ok().and_then(|s| serde_json::from_str(&s).ok()).unwrap_or_default();

    let mut templates = Vec::new();
    if let Ok(entries) = fs::read_dir("templates") {
        for entry in entries.flatten() {
            let path = entry.path();
            if let Some(ext) = path.extension() {
                if ext == "png" {
                    if let Ok(img) = image::open(&path) {
                        let (tw, th) = img.dimensions();
                        let rgb_data = img.to_rgb8().into_raw();
                        let name = path.file_name().unwrap().to_str().unwrap().to_string();
                        templates.push(Template { name, data_rgb: rgb_data, w: tw, h: th });
                    }
                }
            }
        }
    }

    loop {
        if grabber.is_none() {
            unsafe {
                let window_name: Vec<u16> = "LD-1".encode_utf16().chain(std::iter::once(0)).collect();
                if let Ok(parent_hwnd) = FindWindowW(None, PCWSTR(window_name.as_ptr())) {
                    if let Ok(g) = WgcGrabber::new(parent_hwnd) {
                        println!("🚀 Mắt thần KHỞI TẠO (Capture Res: {}x{})", g.get_resolution().0, g.get_resolution().1);
                        grabber = Some(g);
                        target_hwnd = Some(parent_hwnd);
                        render_hwnd_cached = Some(get_game_render_window(parent_hwnd));
                        
                        let mut client_rect = RECT::default();
                        let _ = GetClientRect(render_hwnd_cached.unwrap(), &mut client_rect);
                        last_actual_res = (client_rect.right as u32, client_rect.bottom as u32);
                    }
                } else {
                    static mut LAST_LOG: u64 = 0;
                    let now = Instant::now().elapsed().as_secs();
                    if now - unsafe { LAST_LOG } > 5 {
                        println!("Đang tìm cửa sổ 'LD-1'...");
                        unsafe { LAST_LOG = now; }
                    }
                }
            }
        }

        if let (Some(ref g), Some(parent_h), Some(render_h)) = (grabber.as_ref(), target_hwnd, render_hwnd_cached) {
            let mut client_rect = RECT::default();
            unsafe { let _ = GetClientRect(render_h, &mut client_rect); }
            let actual_w = client_rect.right as u32;
            let actual_h = client_rect.bottom as u32;

            // Resolution Guard: Chỉ recreate grabber để đảm bảo chất lượng ảnh, KHÔNG xóa positions
            if actual_w != 0 && actual_h != 0 && (actual_w != last_actual_res.0 || actual_h != last_actual_res.1) {
                println!("🔄 Resolution Guard: Đổi kích thước cửa sổ ({}x{}) -> ({}x{}).", last_actual_res.0, last_actual_res.1, actual_w, actual_h);
                last_actual_res = (actual_w, actual_h);
                grabber = None;
                sleep(Duration::from_millis(300));
                continue;
            }
            last_actual_res = (actual_w, actual_h);

            if let Ok(frame_bgra) = g.capture_frame() {
                let frame_bgra: Vec<u8> = frame_bgra;
                let (cw, ch) = g.get_resolution();
                
                let mut pt = POINT { x: 0, y: 0 };
                let mut rect = RECT::default();
                unsafe {
                    let _ = ClientToScreen(render_h, &mut pt);
                    let _ = GetWindowRect(parent_h, &mut rect);
                }
                let offset_x = (pt.x - rect.left).max(0) as u32;
                let offset_y = (pt.y - rect.top).max(0) as u32;

                // 1. Chụp ảnh từ cửa sổ hiện tại (Actual size)
                let mut screen_img: RgbaImage = ImageBuffer::new(actual_w, actual_h);
                for y in 0..actual_h {
                    for x in 0..actual_w {
                        let sx = x + offset_x;
                        let sy = y + offset_y;
                        if sx < cw && sy < ch {
                            let idx = (sy * cw + sx) as usize * 4;
                            if idx + 2 < frame_bgra.len() {
                                screen_img.put_pixel(x, y, Rgba([frame_bgra[idx+2], frame_bgra[idx+1], frame_bgra[idx], 255]));
                            }
                        }
                    }
                }

                // 2. CHUẨN HÓA: Resize ảnh về Base Resolution (948x584)
                let start_resize = Instant::now();
                let normalized_img = if actual_w != BASE_W || actual_h != BASE_H {
                    image::imageops::resize(&screen_img, BASE_W, BASE_H, image::imageops::FilterType::Triangle)
                } else {
                    screen_img.clone()
                };
                let dur_resize = start_resize.elapsed();
                
                let screen_rgb = image::DynamicImage::ImageRgba8(normalized_img.clone()).to_rgb8().into_raw();
                let _ = normalized_img.save("debug_view.png");

                for template in &templates {
                    let mut found_pos = None;
                    println!("--- Đang check: {} ---", template.name);

                    let start_roi = Instant::now();
                    let mut roi_success = false;
                    if let Some(pos) = positions.get(&template.name) {
                        let rx = pos.x.saturating_sub(40) as usize;
                        let ry = pos.y.saturating_sub(40) as usize;
                        let rw = (template.w as usize + 80).min(BASE_W as usize - rx);
                        let rh = (template.h as usize + 80).min(BASE_H as usize - ry);
                        
                        let mut roi_rgb = Vec::with_capacity(rw * rh * 3);
                        for y in 0..rh {
                            let start = ((ry + y) * BASE_W as usize + rx) * 3;
                            if start + rw * 3 <= screen_rgb.len() {
                                roi_rgb.extend_from_slice(&screen_rgb[start..start + rw * 3]);
                            }
                        }
                        
                        if roi_rgb.len() == rw * rh * 3 {
                            if let Some((x, y, _)) = FastRecognizer::find_template_step(&roi_rgb, rw, rh, &template.data_rgb, template.w as usize, template.h as usize, 25) {
                                found_pos = Some((rx + x, ry + y));
                                roi_success = true;
                            }
                        }
                    }
                    let dur_roi = start_roi.elapsed();

                    let start_full = Instant::now();
                    let mut full_success = false;
                    if found_pos.is_none() {
                        if let Some((x, y, _)) = FastRecognizer::find_template_step(&screen_rgb, BASE_W as usize, BASE_H as usize, &template.data_rgb, template.w as usize, template.h as usize, 25) {
                            found_pos = Some((x, y));
                            full_success = true;
                        }
                    }
                    let dur_full = start_full.elapsed();

                    let roi_status = if roi_success { "SUCCESS" } else { "FAILED" };
                    let full_status = if full_success { "SUCCESS" } else if found_pos.is_some() { "SKIPPED" } else { "FAILED" };

                    println!("  >> ROI: {}μs ({}) | Full: {}μs ({}) | Resize: {}ms", 
                        dur_roi.as_micros(), roi_status, dur_full.as_micros(), full_status, dur_resize.as_millis());

                    if let Some((fx, fy)) = found_pos {
                        println!("🎯 CLICK: {} tại ({}, {}) [Base]", template.name, fx, fy);
                        positions.insert(template.name.clone(), Pos { x: fx as u32, y: fy as u32 });
                        
                        if let Ok(mut adb) = AdbClient::connect_server("127.0.0.1", 5037) {
                            // Tọa độ luôn tính dựa trên BASE_RESOLUTION sang ANDROID_RESOLUTION
                            let android_w = 960.0f64;
                            let android_h = 540.0f64;
                            let scale_x = android_w / BASE_W as f64;
                            let scale_y = android_h / BASE_H as f64;
                            
                            use rand::Rng;
                            let mut rng = rand::thread_rng();
                            let tap_x = ((fx as f64 + template.w as f64 / 2.0) * scale_x) as i32 + rng.gen_range(-2..3);
                            let tap_y = ((fy as f64 + template.h as f64 / 2.0) * scale_y) as i32 + rng.gen_range(-2..3);
                            
                            let _ = adb.tap("emulator-5556", tap_x, tap_y);
                        }
                    }
                }
                let _ = fs::write("positions.json", serde_json::to_string(&positions).unwrap());
            }
        }
        sleep(Duration::from_millis(5000));
    }
}
