use std::sync::Arc;
use std::fs;
use std::collections::HashMap;
use tauri::State;
use image::{GenericImageView, RgbaImage, ImageBuffer};
use windows::Win32::Foundation::{HWND, RECT, POINT, LPARAM, WPARAM};
use windows::Win32::UI::WindowsAndMessaging::{
    GetClientRect, GetWindowRect, PostMessageW,
    WM_LBUTTONDOWN, WM_LBUTTONUP
};
use windows::Win32::Graphics::Gdi::ClientToScreen;

use crate::config::state::{AppState, CachedTemplate};
use crate::config::persisted::AppConfig;
use crate::core::recognize::FastRecognizer;
use crate::commands::device::get_or_create_grabber;
use crate::automation::utils::find_template_with_variants;

const BASE_W: u32 = 960;
const BASE_H: u32 = 540;

#[tauri::command]
pub fn get_templates(state: State<'_, AppState>) -> Vec<String> {
    let mut names = Vec::new();
    let mut cache = state.template_cache.lock().unwrap();
    cache.clear(); // Làm mới Cache khi quét lại

    // Quét 4 thư mục con theo danh mục
    let categories = ["buttons", "seeds", "seeds/digits", "tools"];
    for cat in &categories {
        let dir_path = format!("templates/{}", cat);
        if let Ok(entries) = fs::read_dir(&dir_path) {
            for entry in entries.flatten() {
                let path = entry.path();
                if let Some(ext) = path.extension() {
                    if ext == "png" {
                        let file_name = path.file_name().unwrap().to_string_lossy().into_owned();
                        let cache_key = format!("{}/{}", cat, file_name);
                        names.push(cache_key.clone());
                        
                        // Nạp vào RAM Cache
                        if let Ok(img) = image::open(&path) {
                            let (w, h) = img.dimensions();
                            let data = Arc::new(img.to_rgb8().into_raw());
                            cache.insert(cache_key, CachedTemplate { dimensions: (w, h), data });
                        }
                    }
                }
            }
        }
    }
    println!("Da nap {} mau vao RAM Cache.", names.len());
    names
}

#[tauri::command]
pub fn capture_screen(state: State<'_, AppState>) -> std::result::Result<String, String> {
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
pub fn test_template(state: State<'_, AppState>, name: String) -> std::result::Result<String, String> {
    let start_total = std::time::Instant::now();

    let (g, _, bind_hwnd, serial) = get_or_create_grabber(&state)?;
    let start_capture = std::time::Instant::now();
    let (aw, ah, screen_img) = unsafe {
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
        (aw, ah, screen_img)
    };
    let capture_time = start_capture.elapsed().as_millis();

    let start_norm = std::time::Instant::now();
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
    let norm_time = start_norm.elapsed().as_millis();

    let max_scan_w = if name.starts_with("seeds/") {
        Some((norm_w / 2) as usize)
    } else {
        None
    };

    // Sử dụng cấu hình độ nhạy khớp động
    let config = AppConfig::load();
    let default_thresh = if config.match_threshold == 0 { 25 } else { config.match_threshold };
    let threshold = if name.starts_with("seeds/") || name.contains("_strict") { 12 } else { default_thresh };

    // Clone templates từ RAM Cache để chạy variant matching
    let templates: HashMap<String, (Arc<Vec<u8>>, u32, u32)> = {
        let mut cache = state.template_cache.lock().unwrap();
        // Nếu cache trống thì nạp lại
        if cache.is_empty() {
            let categories = ["buttons", "seeds", "seeds/digits", "tools"];
            for cat in &categories {
                let dir_path = format!("templates/{}", cat);
                if let Ok(entries) = fs::read_dir(&dir_path) {
                    for entry in entries.flatten() {
                        let path = entry.path();
                        if let Some(ext) = path.extension() {
                            if ext == "png" {
                                let file_name = path.file_name().unwrap().to_string_lossy().into_owned();
                                let cache_key = format!("{}/{}", cat, file_name);
                                if let Ok(img) = image::open(&path) {
                                    let (w, h) = img.dimensions();
                                    let data = Arc::new(img.to_rgb8().into_raw());
                                    cache.insert(cache_key, CachedTemplate { dimensions: (w, h), data });
                                }
                            }
                        }
                    }
                }
            }
        }
        cache.iter().map(|(k, v)| (k.clone(), (Arc::clone(&v.data), v.dimensions.0, v.dimensions.1))).collect()
    };

    let start_recog = std::time::Instant::now();
    if let Some((found_key, fx, fy, tw, th, score)) = find_template_with_variants(&screen_rgba, &templates, &name, threshold, max_scan_w) {
        let recog_time = start_recog.elapsed().as_millis();
        let tx = (fx as f64 + tw as f64 / 2.0) as i32;
        let ty = (fy as f64 + th as f64 / 2.0) as i32;
        
        let actual_tx = (tx as f64 * aw as f64 / norm_w as f64) as i32;
        let actual_ty = (ty as f64 * ah as f64 / norm_h as f64) as i32;
        
        let start_click = std::time::Instant::now();
        // Gui click tuc thoi 0ms
        unsafe {
            let _ = PostMessageW(Some(bind_hwnd), WM_LBUTTONDOWN, WPARAM(1), LPARAM(((actual_ty << 16) | (actual_tx & 0xFFFF)) as isize));
            let _ = PostMessageW(Some(bind_hwnd), WM_LBUTTONUP, WPARAM(0), LPARAM(((actual_ty << 16) | (actual_tx & 0xFFFF)) as isize));
        }
        let click_time = start_click.elapsed().as_millis();
        
        let total_time = start_total.elapsed().as_millis();
        let similarity = (1.0 - (score as f64 / 255.0)) * 100.0;
        logs.push(format!("[KHOP] {} ({}, {}) | Do khop (Similarity): {:.2}% | Sai lech (SAD): {}", found_key, fx, fy, similarity, score));
        logs.push(format!("[WIN32] Click {} tai ({}, {}) (scaled tu {}, {})", serial, actual_tx, actual_ty, tx, ty));
        
        // Gọi hàm chuyên biệt định dạng log thời gian chi tiết
        logs.push(crate::automation::utils::log_time_diagnostics(capture_time, norm_time, recog_time, click_time, total_time));
        Ok(logs.join("\n"))
    } else {
        let recog_time = start_recog.elapsed().as_millis();
        let total_time = start_total.elapsed().as_millis();
        let mut err_msg = format!("Khong tim thay mau! [TIME] Chup anh: {}ms | Chuan hoa: {}ms | Quet: {}ms | Tong: {}ms", capture_time, norm_time, recog_time, total_time);
        if !logs.is_empty() {
            err_msg = format!("{}\n{}", logs.join("\n"), err_msg);
        }
        Err(err_msg)
    }
}

#[tauri::command]
pub fn test_all_templates(state: State<'_, AppState>) -> std::result::Result<String, String> {
    let start_total = std::time::Instant::now();
    
    // 1. Capture 1 frame duy nhất cho thiết bị active
    let (g, _, bind_hwnd, serial) = get_or_create_grabber(&state)?;
    
    let (aw, ah, screen_img) = unsafe {
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
        (aw, ah, screen_img)
    };

    // Chuẩn hóa khung hình duy nhất
    let diff_w = (aw as i32 - BASE_W as i32).abs();
    let diff_h = (ah as i32 - BASE_H as i32).abs();
    let norm = if diff_w > 4 || diff_h > 4 {
        image::imageops::resize(&screen_img, BASE_W, BASE_H, image::imageops::FilterType::Nearest)
    } else {
        screen_img
    };
    
    let norm_w = norm.width();
    let norm_h = norm.height();
    let screen_rgba = norm.into_raw();

    // 2. Lấy toàn bộ các templates trong thư mục hoặc cache
    let templates: Vec<(String, Arc<Vec<u8>>, u32, u32)> = {
        let mut cache = state.template_cache.lock().unwrap();
        // Nếu cache trống thì nạp lại theo danh mục
        if cache.is_empty() {
            let categories = ["buttons", "seeds", "seeds/digits", "tools"];
            for cat in &categories {
                let dir_path = format!("templates/{}", cat);
                if let Ok(entries) = fs::read_dir(&dir_path) {
                    for entry in entries.flatten() {
                        let path = entry.path();
                        if let Some(ext) = path.extension() {
                            if ext == "png" {
                                let file_name = path.file_name().unwrap().to_string_lossy().into_owned();
                                let cache_key = format!("{}/{}", cat, file_name);
                                if let Ok(img) = image::open(&path) {
                                    let (w, h) = img.dimensions();
                                    let data = Arc::new(img.to_rgb8().into_raw());
                                    cache.insert(cache_key, CachedTemplate { dimensions: (w, h), data });
                                }
                            }
                        }
                    }
                }
            }
        }
        cache.iter().map(|(name, t)| (name.clone(), Arc::clone(&t.data), t.dimensions.0, t.dimensions.1)).collect()
    };

    let mut logs = Vec::new();
    logs.push(format!("===== BAT DAU KIEM TRA TOAN BO ANH MAU ({}) =====", serial));
    let mut found_count = 0;

    // Sử dụng cấu hình độ nhạy khớp động
    let config = AppConfig::load();
    let default_thresh = if config.match_threshold == 0 { 25 } else { config.match_threshold };

    for (name, template_data, tw, th) in templates {
        if crate::automation::utils::check_and_clear_cancelled(&state) {
            logs.push("[DUNG] Da dung qua trinh kiem tra do nguoi dung yeu cau.".to_string());
            return Err(logs.join("\n"));
        }
        let max_scan_w = if name.starts_with("seeds/") {
            Some((norm_w / 2) as usize)
        } else {
            None
        };

        let threshold = if name.starts_with("seeds/") || name.contains("_strict") { 12 } else { default_thresh };

        if let Some((fx, fy, score)) = FastRecognizer::find_template_step(
            &screen_rgba, norm_w as usize, norm_h as usize, 4, 
            &template_data, tw as usize, th as usize, threshold, max_scan_w
        ) {
            let tx = (fx as f64 + tw as f64 / 2.0) as i32;
            let ty = (fy as f64 + th as f64 / 2.0) as i32;
            let actual_tx = (tx as f64 * aw as f64 / norm_w as f64) as i32;
            let actual_ty = (ty as f64 * ah as f64 / norm_h as f64) as i32;
            
            logs.push(format!("[TIM THAY] {} -> scaled: ({}, {}), goc: ({}, {}) | Score: {}", name, tx, ty, actual_tx, actual_ty, score));
            found_count += 1;
        } else {
            logs.push(format!("[KHONG THAY] {}", name));
        }
    }

    let elapsed = start_total.elapsed().as_millis();
    logs.push(format!("===== HOAN THANH: Tim thay {}/{} anh mau | Tong: {}ms =====", found_count, logs.len() - 2, elapsed));
    
    Ok(logs.join("\n"))
}

#[tauri::command]
pub async fn check_seeds_templates(state: State<'_, AppState>) -> std::result::Result<String, String> {
    crate::automation::check_seeds::run_check_seeds_templates_logic(&state)
}

#[tauri::command]
pub fn test_digit_recognition(state: State<'_, AppState>) -> std::result::Result<String, String> {
    let start_total = std::time::Instant::now();
    let (g, _, bind_hwnd, _) = get_or_create_grabber(&state)?;
    
    // Capture screen using our standardized helper and measure time
    let start_capture = std::time::Instant::now();
    let (_aw, _ah, screen_rgba) = crate::automation::utils::capture_helper(&g, bind_hwnd, &state)?;
    let capture_time = start_capture.elapsed().as_millis();

    // Map templates into expected HashMap format
    let templates: HashMap<String, (Arc<Vec<u8>>, u32, u32)> = {
        let mut cache = state.template_cache.lock().unwrap();
        if cache.is_empty() {
            let categories = ["buttons", "seeds", "seeds/digits", "tools"];
            for cat in &categories {
                let dir_path = format!("templates/{}", cat);
                if let Ok(entries) = fs::read_dir(&dir_path) {
                    for entry in entries.flatten() {
                        let path = entry.path();
                        if let Some(ext) = path.extension() {
                            if ext == "png" {
                                let file_name = path.file_name().unwrap().to_string_lossy().into_owned();
                                let cache_key = format!("{}/{}", cat, file_name);
                                if let Ok(img) = image::open(&path) {
                                    let (w, h) = img.dimensions();
                                    let raw_data = Arc::new(img.to_rgb8().into_raw());
                                    cache.insert(cache_key, CachedTemplate {
                                        data: raw_data,
                                        dimensions: (w, h),
                                    });
                                }
                            }
                        }
                    }
                }
            }
        }
        cache.iter().map(|(name, t)| (name.clone(), (Arc::clone(&t.data), t.dimensions.0, t.dimensions.1))).collect()
    };

    let slider_key = "buttons/slider.png";
    let slider_y = if let Some((_, _fx, fy, _w, _h, _score)) = crate::automation::utils::find_template_with_variants(
        &screen_rgba, &templates, slider_key, 25, None
    ) {
        fy as usize + _h as usize / 2
    } else {
        340
    };

    let start_scan = std::time::Instant::now();
    let recognized = crate::automation::utils::recognize_number_above_slider(&screen_rgba, &templates, slider_y);
    let scan_time = start_scan.elapsed().as_millis();
    
    let total_time = start_total.elapsed().as_millis();
    
    // Call our specialized diagnostic time logging helper
    let time_log = crate::automation::utils::log_time_diagnostics(
        capture_time,
        0,
        scan_time,
        0,
        total_time
    );

    let mut template_sizes = Vec::new();
    for digit in 0..=9 {
        let key = format!("seeds/digits/{}.png", digit);
        if let Some((_, tw, th)) = templates.get(&key) {
            template_sizes.push(format!("{}:{}x{}", digit, tw, th));
        }
    }
    let template_sizes_str = template_sizes.join(", ");

    if let Some(recognized_val) = recognized {
        Ok(format!(
            "[TEST SỐ] Vùng Y quét: {} | Nhận dạng: {} hạt | Kích thước ảnh mẫu: [{}]\n{}",
            slider_y, recognized_val, template_sizes_str, time_log
        ))
    } else {
        Ok(format!(
            "[TEST SỐ] Vùng Y quét: {} | KHÔNG NHẬN DẠNG ĐƯỢC CHỮ SỐ NÀO trong vùng 120x35 (x: 420..540, y: {}..{}) | Kích thước ảnh mẫu: [{}]\n{}",
            slider_y,
            slider_y.saturating_sub(45),
            slider_y.saturating_sub(10),
            template_sizes_str,
            time_log
        ))
    }
}

#[tauri::command]
pub fn get_seed_names() -> HashMap<String, String> {
    let path = "templates/seeds/names.json";
    if let Ok(content) = fs::read_to_string(path) {
        if let Ok(map) = serde_json::from_str::<HashMap<String, String>>(&content) {
            return map;
        }
    }
    HashMap::new()
}

#[tauri::command]
pub fn get_tool_names() -> HashMap<String, String> {
    let path = "templates/tools/names.json";
    if let Ok(content) = fs::read_to_string(path) {
        if let Ok(map) = serde_json::from_str::<HashMap<String, String>>(&content) {
            return map;
        }
    }
    HashMap::new()
}
