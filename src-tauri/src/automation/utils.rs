use std::time::Duration;
use std::thread::sleep;
use std::collections::HashMap;
use std::sync::Arc;
use tauri::State;
use crate::core::recognize::FastRecognizer;
use image::{ImageBuffer, RgbaImage};
use windows::Win32::Foundation::{HWND, LPARAM, WPARAM};
use windows::Win32::UI::WindowsAndMessaging::{
    GetClientRect, GetWindowRect, PostMessageW, WM_LBUTTONDOWN, WM_LBUTTONUP, WM_MOUSEMOVE
};
use windows::Win32::Graphics::Gdi::ClientToScreen;
use windows::Win32::Foundation::POINT;
use windows::Win32::Foundation::RECT;

use crate::config::state::AppState;
use crate::core::capture::WgcGrabber;

pub const BASE_W: u32 = 960;
pub const BASE_H: u32 = 540;

pub fn check_and_clear_cancelled(state: &State<'_, AppState>) -> bool {
    if let Some(device) = state.active_device.lock().unwrap().as_ref() {
        let mut cancelled = state.cancelled_devices.lock().unwrap();
        if cancelled.contains(&device.handle) {
            cancelled.remove(&device.handle);
            return true;
        }
    }
    false
}

pub fn swipe_ld(bind_hwnd: HWND, aw: u32, ah: u32, x1: i32, y1: i32, x2: i32, y2: i32) {
    let actual_x1 = (x1 as f64 * aw as f64 / BASE_W as f64) as i32;
    let actual_y1 = (y1 as f64 * ah as f64 / BASE_H as f64) as i32;
    let actual_x2 = (x2 as f64 * aw as f64 / BASE_W as f64) as i32;
    let actual_y2 = (y2 as f64 * ah as f64 / BASE_H as f64) as i32;

    let lparam_start = ((actual_y1 << 16) | (actual_x1 & 0xFFFF)) as isize;
    let _ = unsafe { PostMessageW(Some(bind_hwnd), WM_LBUTTONDOWN, WPARAM(1), LPARAM(lparam_start)) };
    sleep(Duration::from_millis(50));

    let steps = 10;
    for i in 1..=steps {
        let curr_x = actual_x1 + (actual_x2 - actual_x1) * i / steps;
        let curr_y = actual_y1 + (actual_y2 - actual_y1) * i / steps;
        let lparam_move = ((curr_y << 16) | (curr_x & 0xFFFF)) as isize;
        let _ = unsafe { PostMessageW(Some(bind_hwnd), WM_MOUSEMOVE, WPARAM(1), LPARAM(lparam_move)) };
        sleep(Duration::from_millis(20));
    }

    let lparam_end = ((actual_y2 << 16) | (actual_x2 & 0xFFFF)) as isize;
    let _ = unsafe { PostMessageW(Some(bind_hwnd), WM_LBUTTONUP, WPARAM(0), LPARAM(lparam_end)) };
    sleep(Duration::from_millis(200));
}

pub fn click_ld(bind_hwnd: HWND, aw: u32, ah: u32, tx: i32, ty: i32) {
    let actual_tx = (tx as f64 * aw as f64 / BASE_W as f64) as i32;
    let actual_ty = (ty as f64 * ah as f64 / BASE_H as f64) as i32;
    unsafe {
        let _ = PostMessageW(Some(bind_hwnd), WM_LBUTTONDOWN, WPARAM(1), LPARAM(((actual_ty << 16) | (actual_tx & 0xFFFF)) as isize));
        let _ = PostMessageW(Some(bind_hwnd), WM_LBUTTONUP, WPARAM(0), LPARAM(((actual_ty << 16) | (actual_tx & 0xFFFF)) as isize));
    }
}

pub fn capture_helper(g: &WgcGrabber, bind_hwnd: HWND, state: &State<'_, AppState>) -> std::result::Result<(u32, u32, Vec<u8>), String> {
    if check_and_clear_cancelled(state) {
        return Err("Da dung kich ban do nguoi dung yeu cau.".to_string());
    }
    unsafe {
        let mut c_rect = RECT::default();
        let _ = GetClientRect(bind_hwnd, &mut c_rect);
        let aw = c_rect.right as u32;
        let ah = c_rect.bottom as u32;
        let frame = g.capture_frame().map_err(|e| format!("{:?}", e))?;
        let (cw, ch) = g.get_resolution();

        let mut pt = POINT::default();
        let mut rect = RECT::default();
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

        let diff_w = (aw as i32 - BASE_W as i32).abs();
        let diff_h = (ah as i32 - BASE_H as i32).abs();
        let norm = if diff_w > 4 || diff_h > 4 {
            image::imageops::resize(&screen_img, BASE_W, BASE_H, image::imageops::FilterType::Nearest)
        } else {
            screen_img
        };
        Ok((aw, ah, norm.into_raw()))
    }
}

pub fn log_time_diagnostics(
    capture_ms: u128,
    normalize_ms: u128,
    scan_ms: u128,
    click_ms: u128,
    total_ms: u128,
) -> String {
    // Để sau muốn tắt hoặc rút gọn log chi tiết, chỉ cần sửa đổi tại hàm duy nhất này!
    // Ví dụ nếu muốn tắt chi tiết và chỉ hiện tổng thời gian:
    // return format!("[TIME] Tong: {}ms", total_ms);

    // Đang bật chế độ chi tiết nhất để chẩn đoán:
    format!(
        "[TIME] Chup anh (Capture): {}ms | Chuan hoa (Normalize): {}ms | Quet (Scan): {}ms | Gui Click (Win32): {}ms | Tong (Total): {}ms",
        capture_ms, normalize_ms, scan_ms, click_ms, total_ms
    )
}

pub fn find_template_with_variants(
    screen_rgba: &[u8],
    templates: &HashMap<String, (Arc<Vec<u8>>, u32, u32)>,
    target_key: &str,
    threshold: u32,
    max_scan_w: Option<usize>,
) -> Option<(String, usize, usize, u32, u32, u32)> { // Trả về: (Khóa_tìm_thấy, x, y, w, h, score)
    let mut variants = Vec::new();
    if let Some(slash_idx) = target_key.find('/') {
        let cat = &target_key[..slash_idx];
        let base = &target_key[slash_idx + 1..];

        for (k, v) in templates {
            if k == target_key {
                variants.push((k.clone(), Arc::clone(&v.0), v.1, v.2));
            } else if k.starts_with(cat) && k.ends_with(base) {
                let remaining = &k[cat.len() + 1..k.len() - base.len()];
                // Khớp mọi tiền tố kết thúc bằng "_" như: "1_", "2_", "new_" v.v.
                if remaining.ends_with('_') {
                    variants.push((k.clone(), Arc::clone(&v.0), v.1, v.2));
                }
            }
        }
    }
    if variants.is_empty() {
        if let Some(v) = templates.get(target_key) {
            variants.push((target_key.to_string(), Arc::clone(&v.0), v.1, v.2));
        }
    }

    // Quét lần lượt từng biến thể trên ảnh chụp màn hình
    for (variant_key, t_data, t_w, t_h) in &variants {
        if let Some((x, y, score)) = FastRecognizer::find_template_step(
            screen_rgba, BASE_W as usize, BASE_H as usize, 4,
            t_data, *t_w as usize, *t_h as usize, threshold, max_scan_w
        ) {
            return Some((variant_key.clone(), x, y, *t_w, *t_h, score));
        }
    }
    None
}

pub fn recognize_number_above_slider(
    screen_rgba: &[u8],
    templates: &HashMap<String, (Arc<Vec<u8>>, u32, u32)>,
    slider_y: usize,
) -> Option<u32> {
    let mut found_digits = Vec::new();

    // Bounding Box above the slider
    // The number is centered horizontally around 480, so we scan x from 420 to 540 (width 120).
    // Vertically, we scan exactly from slider_y - 45 to slider_y - 10 (height 35).
    let min_y = slider_y.saturating_sub(45);
    let max_y = slider_y.saturating_sub(10);

    let sub_x = 420;
    let sub_w = 120;
    let sub_h = max_y.saturating_sub(min_y);
    if sub_h == 0 { return None; }

    let mut sub_rgba = vec![0u8; sub_w * sub_h * 4];
    for y in 0..sub_h {
        let sy = min_y + y;
        if sy >= BASE_H as usize { continue; }
        let src_start = (sy * BASE_W as usize + sub_x) * 4;
        let dest_start = y * sub_w * 4;
        if src_start + sub_w * 4 <= screen_rgba.len() {
            sub_rgba[dest_start..dest_start + sub_w * 4].copy_from_slice(&screen_rgba[src_start..src_start + sub_w * 4]);
        }
    }

    // Now scan for each digit 0..=9 inside the sub-image!
    for digit in 0..=9 {
        let key = format!("seeds/digits/{}.png", digit);
        if let Some((t_data, tw, th)) = templates.get(&key) {
            let mut start_x = 0;
            while start_x + *tw as usize <= sub_w {
                if let Some((fx, _fy, _score)) = find_digit_in_subimage(
                    &sub_rgba, sub_w, sub_h,
                    t_data, *tw as usize, *th as usize,
                    24, start_x
                ) {
                    let actual_x = sub_x + fx;
                    found_digits.push((actual_x, digit));
                    start_x = fx + (*tw as usize).max(1); // Move past the matched digit
                } else {
                    break;
                }
            }
        }
    }

    if found_digits.is_empty() {
        return None;
    }

    // Sort from left to right
    found_digits.sort_by_key(|&(x, _)| x);

    // Reconstruct the number
    let mut number = 0;
    for &(_, digit) in &found_digits {
        number = number * 10 + digit;
    }
    Some(number)
}

fn find_digit_in_subimage(
    sub_rgba: &[u8], sub_w: usize, sub_h: usize,
    tpl_rgb: &[u8], tpl_w: usize, tpl_h: usize,
    threshold: u32, start_x: usize
) -> Option<(usize, usize, u32)> {
    if tpl_w > sub_w || tpl_h > sub_h { return None; }

    for y in 0..sub_h.saturating_sub(tpl_h) {
        for x in start_x..sub_w.saturating_sub(tpl_w) {
            let mut quick_possible = true;
            let inset_x = (tpl_w / 6).max(1);
            let inset_y = (tpl_h / 6).max(1);
            let test_points = [
                (inset_x, inset_y),
                (tpl_w - 1 - inset_x, inset_y),
                (inset_x, tpl_h - 1 - inset_y),
                (tpl_w - 1 - inset_x, tpl_h - 1 - inset_y),
                (tpl_w / 2, tpl_h / 2),
            ];

            for (tx, ty) in test_points {
                let img_idx = ((y + ty) * sub_w + (x + tx)) * 4;
                let tpl_idx = (ty * tpl_w + tx) * 3;
                if img_idx + 2 < sub_rgba.len() && tpl_idx + 2 < tpl_rgb.len() {
                    let img_gray = (sub_rgba[img_idx] as i32 + sub_rgba[img_idx+1] as i32 + sub_rgba[img_idx+2] as i32) / 3;
                    let tpl_gray = (tpl_rgb[tpl_idx] as i32 + tpl_rgb[tpl_idx+1] as i32 + tpl_rgb[tpl_idx+2] as i32) / 3;

                    if (img_gray - tpl_gray).abs() as u32 > threshold * 2 {
                        quick_possible = false;
                        break;
                    }
                }
            }

            if quick_possible {
                let mut total_sad: u64 = 0;
                let mut count = 0;

                for ty in 0..tpl_h {
                    for tx in 0..tpl_w {
                        let img_idx = ((y + ty) * sub_w + (x + tx)) * 4;
                        let tpl_idx = (ty * tpl_w + tx) * 3;
                        if img_idx + 2 < sub_rgba.len() && tpl_idx + 2 < tpl_rgb.len() {
                            let r_diff = (sub_rgba[img_idx] as i32 - tpl_rgb[tpl_idx] as i32).abs();
                            let g_diff = (sub_rgba[img_idx+1] as i32 - tpl_rgb[tpl_idx+1] as i32).abs();
                            let b_diff = (sub_rgba[img_idx+2] as i32 - tpl_rgb[tpl_idx+2] as i32).abs();

                            let diff = (r_diff + g_diff + b_diff) / 3;
                            total_sad += diff as u64;
                            count += 1;
                        }
                    }
                }

                let score = (total_sad / count.max(1)) as u32;
                if score < threshold {
                    return Some((x, y, score));
                }
            }
        }
    }
    None
}

pub fn find_and_click_with_timeout(
    state: &State<'_, AppState>,
    g: &WgcGrabber,
    bind_hwnd: HWND,
    templates: &HashMap<String, (Arc<Vec<u8>>, u32, u32)>,
    key: &str,
    name_vi: &str,
    timeout: Duration,
    threshold: u32,
    click_delay: Duration,
    must_exist: bool,
    add_log: &dyn Fn(String),
) -> Result<bool, String> {
    let start = std::time::Instant::now();
    add_log(format!("Dang tim nut [{}] ({})...", key, name_vi));
    while start.elapsed() < timeout {
        if check_and_clear_cancelled(state) {
            return Err("Da dung kich ban.".to_string());
        }

        let (aw, ah, screen_rgba) = capture_helper(g, bind_hwnd, state)
            .map_err(|e| format!("Loi chup man hinh: {}", e))?;

        if let Some((_, fx, fy, tw, th, _)) = find_template_with_variants(
            &screen_rgba, templates, key, threshold, None
        ) {
            let tx = (fx as f64 + tw as f64 / 2.0) as i32;
            let ty = (fy as f64 + th as f64 / 2.0) as i32;
            click_ld(bind_hwnd, aw, ah, tx, ty);
            add_log(format!("-> Da click [{}] tai ({}, {})", key, tx, ty));

            // Bỏ qua đối thoại NPC bằng cách click 3 lần (mỗi lần cách nhau 300ms) đối với các nút được chỉ định
            let is_close_shop = key == "buttons/close-harvest.png" && name_vi == "Dong cua hang ban";
            if key == "buttons/open-farm-shop.png" || key == "buttons/open-seed-shop.png" ||
               key == "buttons/open-tool-shop.png" ||
               key == "buttons/close-seed.png" || key == "buttons/leave.png" || is_close_shop {
                add_log(format!("[BYPASS NPC] Thuc hien click 3 lan tai ({}, {}) voi chu ky 300ms...", tx, ty));
                for _ in 0..2 {
                    if check_and_clear_cancelled(state) {
                        return Err("Da dung kich ban.".to_string());
                    }
                    std::thread::sleep(Duration::from_millis(300));
                    click_ld(bind_hwnd, aw, ah, tx, ty);
                }
            }

            // Sleep click delay
            let sleep_start = std::time::Instant::now();
            while sleep_start.elapsed() < click_delay {
                if check_and_clear_cancelled(state) {
                    return Err("Da dung kich ban.".to_string());
                }
                std::thread::sleep(Duration::from_millis(20));
            }
            return Ok(true);
        }
        std::thread::sleep(Duration::from_millis(300));
    }

    if must_exist {
        add_log(format!("[LOI] Qua thoi gian cho nhung khong tim thay nut [{}]", key));
        Err(format!("Qua thoi gian cho nut {}", name_vi))
    } else {
        add_log(format!("Khong tim thay nut [{}], bo qua.", key));
        Ok(false)
    }
}

pub fn is_game_running(index: i32) -> bool {
    let ld_path = crate::emulator::ldplayer::get_ld_path();
    use std::os::windows::process::CommandExt;
    let output = std::process::Command::new(&ld_path)
        .args(&[
            "adb",
            "--index",
            &index.to_string(),
            "--command",
            "shell pidof com.vng.playtogether",
        ])
        .creation_flags(0x08000000) // CREATE_NO_WINDOW
        .output();

    if let Ok(out) = output {
        let stdout = String::from_utf8_lossy(&out.stdout).trim().to_string();
        !stdout.is_empty()
    } else {
        false
    }
}

pub fn launch_game(index: i32) -> std::result::Result<(), String> {
    let ld_path = crate::emulator::ldplayer::get_ld_path();
    use std::os::windows::process::CommandExt;
    let output = std::process::Command::new(&ld_path)
        .args(&[
            "runapp",
            "--index",
            &index.to_string(),
            "--packagename",
            "com.vng.playtogether",
        ])
        .creation_flags(0x08000000) // CREATE_NO_WINDOW
        .output();
    
    if let Err(e) = output {
        return Err(format!("Lỗi khi mở game: {:?}", e));
    }
    Ok(())
}

pub fn ensure_game_ready(
    state: &State<'_, AppState>,
    add_log: &dyn Fn(String)
) -> std::result::Result<(), String> {
    // 1. Lấy thông tin thiết bị đang hoạt động
    let device = {
        let active = state.active_device.lock().unwrap();
        match active.as_ref() {
            Some(d) => d.clone(),
            None => return Err("Không có thiết bị hoạt động nào được chọn.".to_string()),
        }
    };

    let config = crate::config::persisted::AppConfig::load();

    // Định nghĩa macro sleep cục bộ để sử dụng ngắt quãng nhanh khi có cờ hủy
    let local_sleep = |dur: Duration| -> std::result::Result<(), String> {
        let start = std::time::Instant::now();
        let step = std::time::Duration::from_millis(20);
        while start.elapsed() < dur {
            if check_and_clear_cancelled(state) {
                return Err("Đã dừng kịch bản do người dùng yêu cầu.".to_string());
            }
            std::thread::sleep(step);
        }
        Ok(())
    };

    // 2. Kiểm tra xem game có đang chạy không
    add_log("[AUTO LOGIN] Đang kiểm tra trạng thái game com.vng.playtogether...".to_string());
    if is_game_running(device.index) {
        add_log("[AUTO LOGIN] Game đã mở sẵn. Tiếp tục chạy kịch bản chính...".to_string());
        return Ok(());
    }

    // 3. Nếu game đang tắt -> Khởi chạy lại game
    add_log("[AUTO LOGIN] Phát hiện game đang tắt. Tiến hành khởi chạy lại game...".to_string());
    launch_game(device.index)?;

    // 4. "xong chờ game_launch_delay_secs"
    let launch_delay = config.game_launch_delay_secs.max(5);
    add_log(format!("[AUTO LOGIN] Đã phát lệnh mở game. Chờ {} giây để game load logo...", launch_delay));
    local_sleep(Duration::from_secs(launch_delay))?;

    // 5. "sau launch_delay sẽ check 1s 1 lần trong 30 giây để tìm nút phone.png"
    add_log("[AUTO LOGIN] Bắt đầu quét tìm nút phone.png (chu kỳ 1s/lần trong tối đa 30 giây)...".to_string());

    let (g, _, bind_hwnd, _) = crate::commands::get_or_create_grabber(state)?;
    
    // Nạp threshold và click delay từ cấu hình
    let threshold = if config.match_threshold == 0 { 25 } else { config.match_threshold };
    let click_delay = Duration::from_millis(if config.click_delay_ms == 0 { 1000 } else { config.click_delay_ms });
    
    // Nạp và cache các mẫu hình ảnh thành định dạng tìm kiếm
    use image::GenericImageView;
    let templates: HashMap<String, (Arc<Vec<u8>>, u32, u32)> = {
        let mut cache = state.template_cache.lock().unwrap();
        if cache.is_empty() {
            let categories = ["buttons", "seeds", "seeds/digits"];
            for cat in &categories {
                let dir_path = format!("templates/{}", cat);
                if let Ok(entries) = std::fs::read_dir(&dir_path) {
                    for entry in entries.flatten() {
                        let path = entry.path();
                        if let Some(ext) = path.extension() {
                            if ext == "png" {
                                let file_name = path.file_name().unwrap().to_string_lossy().into_owned();
                                let cache_key = format!("{}/{}", cat, file_name);
                                if let Ok(img) = image::open(&path) {
                                    let (w, h) = img.dimensions();
                                    let raw_data = Arc::new(img.to_rgb8().into_raw());
                                    cache.insert(cache_key, crate::config::state::CachedTemplate {
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

    // 6. Tìm và click phone.png (chờ 30 giây)
    let found_phone = find_and_click_with_timeout(
        state, &g, bind_hwnd, &templates,
        "buttons/phone.png", "Nút điện thoại",
        Duration::from_secs(30), threshold, click_delay, true, add_log
    )?;

    if !found_phone {
        return Err("[AUTO LOGIN] Không tìm thấy nút phone.png sau 30 giây quét chờ đăng nhập.".to_string());
    }

    // 7. Tìm và click go-home.png (chờ cấu hình button_timeout_secs, tối thiểu 5 giây)
    let go_home_timeout = Duration::from_secs(config.button_timeout_secs.max(5) as u64);
    let found_go_home = find_and_click_with_timeout(
        state, &g, bind_hwnd, &templates,
        "buttons/go-home.png", "Nút về nhà",
        go_home_timeout, threshold, click_delay, true, add_log
    )?;

    if !found_go_home {
        return Err("[AUTO LOGIN] Không tìm thấy nút go-home.png sau khi đã click phone.png.".to_string());
    }

    // "chờ 20s và tiếp tục thực thi các kịch bản"
    add_log("[AUTO LOGIN] Đã click teleport về nhà. Chờ 20 giây để nhân vật load hoàn tất về sân...".to_string());
    local_sleep(Duration::from_secs(20))?;

    add_log("[AUTO LOGIN] Đã hoàn thành quy trình Auto-Login thành công! Bắt đầu kịch bản chính...".to_string());
    Ok(())
}
