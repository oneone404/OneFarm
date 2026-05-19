use std::sync::Arc;
use std::collections::HashMap;
use std::fs;
use std::time::{Duration, Instant};
use tauri::State;
use image::GenericImageView;

use crate::config::state::{AppState, CachedTemplate};
use crate::config::persisted::AppConfig;
use crate::commands::get_or_create_grabber;
use crate::automation::utils::{
    check_and_clear_cancelled, capture_helper, find_template_with_variants, click_ld, BASE_W
};

pub fn run_harvest_sell_script_logic(state: &State<'_, AppState>) -> std::result::Result<String, String> {
    use std::cell::RefCell;
    let logs = RefCell::new(Vec::new());
    let add_log = |msg: String| {
        println!("{}", msg);
        logs.borrow_mut().push(msg);
    };

    macro_rules! sleep {
        ($dur:expr) => {
            let start = Instant::now();
            let step = Duration::from_millis(20);
            while start.elapsed() < $dur {
                if check_and_clear_cancelled(state) {
                    let err_log = logs.borrow().join("\n");
                    return Err(err_log);
                }
                std::thread::sleep(step);
            }
        };
    }

    let (g, _, bind_hwnd, serial) = get_or_create_grabber(state)?;

    let config = AppConfig::load();
    let timeout_secs = if config.button_timeout_secs == 0 { 5 } else { config.button_timeout_secs };
    let step_timeout = Duration::from_secs(timeout_secs);
    let click_delay = Duration::from_millis(if config.click_delay_ms == 0 { 1000 } else { config.click_delay_ms });
    let threshold = if config.match_threshold == 0 { 25 } else { config.match_threshold };

    add_log(format!("===== BAT DAU KICH BAN THU HOACH & BAN ({}) =====", serial));

    // Thực hiện tự động kiểm tra và đăng nhập game (Auto-Login check nếu cấu hình được bật)
    if config.enable_auto_login {
        if let Err(e) = crate::automation::utils::ensure_game_ready(state, &add_log) {
            add_log(format!("[THAT BAI] Lỗi Auto-Login: {}", e));
            return Err(logs.borrow().join("\n"));
        }
    }

    // 1. Nạp và cache các mẫu hình ảnh
    let templates: HashMap<String, (Arc<Vec<u8>>, u32, u32)> = {
        let mut cache = state.template_cache.lock().unwrap();
        if cache.is_empty() {
            let categories = ["buttons", "seeds", "seeds/digits"];
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

    // Định nghĩa helper tìm và click một nút bấm sử dụng hàm dùng chung toàn cục từ utils.rs để tránh lặp code
    let find_and_click_with_timeout = |key: &str, name_vi: &str, must_exist: bool| -> Result<bool, String> {
        crate::automation::utils::find_and_click_with_timeout(
            state, &g, bind_hwnd, &templates, key, name_vi,
            step_timeout, threshold, click_delay, must_exist, &add_log
        )
    };

    // Hàm đếm số lượng ảnh trùng khớp trên màn hình bằng cách xoá vùng đã nhận diện để tiếp tục quét
    let count_template_occurrences = |key: &str, screen_rgba: &[u8]| -> usize {
        let mut count = 0;
        let mut temp_screen = screen_rgba.to_vec();
        while let Some((_, x, y, w, h, _)) = find_template_with_variants(
            &temp_screen, &templates, key, threshold, None
        ) {
            count += 1;
            for row in y..(y + h as usize) {
                for col in x..(x + w as usize) {
                    let idx = (row * BASE_W as usize + col) * 4;
                    if idx + 3 < temp_screen.len() {
                        temp_screen[idx] = 0;
                        temp_screen[idx+1] = 0;
                        temp_screen[idx+2] = 0;
                        temp_screen[idx+3] = 0;
                    }
                }
            }
        }
        count
    };

    // ─── BƯỚC 1: KIỂM TRA VÀ TẮT MODAL MUA HẠT (NẾU ĐANG MỞ) ─────────────────────
    let (_aw, _ah, screen_rgba) = capture_helper(&g, bind_hwnd, state)
        .map_err(|e| format!("Loi chup man hinh check gift: {}", e))?;
    let is_seed_modal_open = find_template_with_variants(
        &screen_rgba, &templates, "buttons/gift.png", threshold, None
    ).is_some();

    if is_seed_modal_open {
        add_log("Phat hien dang o modal mua hat. Tien hanh thoat...".to_string());
        find_and_click_with_timeout("buttons/close-seed.png", "Dong modal mua hat", true)?;

        // Hiện thêm 1 modal trò chuyện, click leave.png và leave2.png
        find_and_click_with_timeout("buttons/leave.png", "Dong hop thoai thoat 1", true)?;
        // Đợi tối đa 1 giây để kiểm tra và click leave2.png nếu có, không bắt buộc tồn tại
        crate::automation::utils::find_and_click_with_timeout(
            state, &g, bind_hwnd, &templates, "buttons/leave2.png", "Dong hop thoai thoat 2",
            Duration::from_secs(1), threshold, click_delay, false, &add_log
        )?;
    } else {
        add_log("Khong o modal mua hat, bo qua buoc tat.".to_string());
    }

    // ─── VÒNG LẶP TUẦN HOÀN: THU HOẠCH & BÁN CHO TỚI KHI HẾT QUẢ ──────────────────
    let mut run_count = 0;
    loop {
        run_count += 1;
        if run_count > 20 {
            add_log("Vuot qua gioi han 20 chu ky Thu hoach & Ban an toan. Dung kich ban.".to_string());
            break;
        }

        add_log(format!("===== CHU KY THU HOACH & BAN LAN {} =====", run_count));

        // ─── BƯỚC 2: QUY TRÌNH DI CHUYỂN VỀ NHÀ VÀ MỞ THU HOẠCH ───────────────────────
        find_and_click_with_timeout("buttons/home.png", "Nut ve nha", true)?;
        find_and_click_with_timeout("buttons/open-house-manage.png", "Nut quan ly nha", true)?;
        find_and_click_with_timeout("buttons/open-harvest.png", "Nut mo thu hoach trai", true)?;

        // ─── BƯỚC 3: VÒNG LẶP THU HOẠCH TRONG CHU KỲ ──────────────────────────────────
        let mut harvest_success_count = 0;
        let mut bag_is_full = false;
        let mut has_more_fruit = true;

        // Chụp ảnh ban đầu để đếm số lượng nút single-harvest.png màu xanh lá
        let (_aw, _ah, s_rgba) = capture_helper(&g, bind_hwnd, state)
            .map_err(|e| format!("Loi chup check so luong trai: {}", e))?;
        let initial_count = count_template_occurrences("buttons/single-harvest.png", &s_rgba);
        add_log(format!("-> Kiem tra ban dau: phat hien {} nut [single-harvest.png] tren man hinh.", initial_count));

        if initial_count < 8 {
            add_log("So nut < 8 -> Da het sach trai thuong (chi con trai khoa). Bo qua thu hoach luot nay.".to_string());
            has_more_fruit = false;
        } else {
            add_log("So nut >= 8 -> Van con trai thuong de thu hoach. Tien hanh thu hoach...".to_string());
            add_log(format!("Bat dau vong lap thu hoach voi so lan: {}", config.harvest_loop_count));

            for i in 1..=config.harvest_loop_count {
                if check_and_clear_cancelled(state) {
                    let err_log = logs.borrow().join("\n");
                    return Err(err_log);
                }
                add_log(format!("--- Luot thu hoach {}/{} ---", i, config.harvest_loop_count));

                // Tìm và click nút harvest.png (Thu hoạch tất cả)
                let mut harvest_found = false;
                let start = Instant::now();
                while start.elapsed() < step_timeout {
                    let (aw, ah, screen_rgba) = capture_helper(&g, bind_hwnd, state)
                        .map_err(|e| format!("Loi chup man hinh harvest: {}", e))?;
                    if let Some((_, fx, fy, tw, th, _)) = find_template_with_variants(
                        &screen_rgba, &templates, "buttons/harvest.png", threshold, None
                    ) {
                        let tx = (fx as f64 + tw as f64 / 2.0) as i32;
                        let ty = (fy as f64 + th as f64 / 2.0) as i32;
                        click_ld(bind_hwnd, aw, ah, tx, ty);
                        add_log(format!("-> Da click harvest tai ({}, {})", tx, ty));
                        sleep!(click_delay);
                        harvest_found = true;
                        break;
                    }
                    sleep!(Duration::from_millis(200));
                }

                if !harvest_found {
                    add_log("Khong tim thay nut [harvest.png]. Dung thu hoach.".to_string());
                    bag_is_full = false;
                    has_more_fruit = false;
                    break;
                }

                // Chờ đợi xem bảng xác nhận confirm.png xuất hiện hay không trong tối đa 1 giây
                let mut confirm_found = false;
                let start_confirm = Instant::now();
                let confirm_timeout = Duration::from_secs(1);
                while start_confirm.elapsed() < confirm_timeout {
                    let (aw, ah, screen_rgba) = capture_helper(&g, bind_hwnd, state)
                        .map_err(|e| format!("Loi chup man hinh confirm: {}", e))?;
                    if let Some((_, fx, fy, tw, th, _)) = find_template_with_variants(
                        &screen_rgba, &templates, "buttons/confirm.png", threshold, None
                    ) {
                        let tx = (fx as f64 + tw as f64 / 2.0) as i32;
                        let ty = (fy as f64 + th as f64 / 2.0) as i32;
                        click_ld(bind_hwnd, aw, ah, tx, ty);
                        add_log(format!("-> Da click confirm tai ({}, {})", tx, ty));
                        sleep!(click_delay);
                        confirm_found = true;
                        break;
                    }
                    sleep!(Duration::from_millis(100));
                }

                if !confirm_found {
                    add_log("Khong thay confirm.png sau khi click harvest.png. Tien hanh check lai so luong trai...".to_string());
                    let (_aw, _ah, s_rgba_check) = capture_helper(&g, bind_hwnd, state)
                        .map_err(|e| format!("Loi chup check lai so luong trai: {}", e))?;
                    let current_count = count_template_occurrences("buttons/single-harvest.png", &s_rgba_check);
                    add_log(format!("-> Phat hien {} nut [single-harvest.png] hien tai.", current_count));

                    if current_count < 8 {
                        add_log("So nut < 8 -> Da thu hoach het trai thuong (chi con trai khoa). Dung thu hoach.".to_string());
                        bag_is_full = false;
                        has_more_fruit = false;
                    } else {
                        add_log("So nut >= 8 nhung khong the thu hoach tiep -> CHẮC CHẮN DO [TÚI ĐẦY]!".to_string());
                        bag_is_full = true;
                        has_more_fruit = true;
                    }
                    break;
                }

                harvest_success_count += 1;
            }
        }

        add_log(format!("Da hoan thanh {} luot thu hoach o chu ky nay.", harvest_success_count));

        // Tắt modal thu hoạch
        find_and_click_with_timeout("buttons/close-harvest.png", "Nut dong thu hoach", true)?;

        // Nếu không có cả túi đầy lẫn quả mới và lượt thu hoạch thành công bằng 0 -> kết thúc kịch bản ngay
        if !bag_is_full && !has_more_fruit && harvest_success_count == 0 {
            add_log("Khong thu hoach duoc gi va da het sach qua. Ket thuc kich ban.".to_string());
            break;
        }

        // ─── BƯỚC 4: BÁN NÔNG SẢN ───────────────────────────────────────────────────
        find_and_click_with_timeout("buttons/farm-shop.png", "Nut di den cua hang ban", true)?;
        find_and_click_with_timeout("buttons/open-farm-shop.png", "Mo cua hang ban 1", true)?;
        find_and_click_with_timeout("buttons/open-farm-shop2.png", "Mo cua hang ban 2", true)?;

        add_log(format!("Bat dau vong lap ban nong san voi so lan: {}", config.sell_loop_count));
        let mut sell_success_count = 0;

        for i in 1..=config.sell_loop_count {
            if check_and_clear_cancelled(state) {
                let err_log = logs.borrow().join("\n");
                return Err(err_log);
            }
            add_log(format!("--- Luot ban {}/{} ---", i, config.sell_loop_count));

            // Click auto_pick.png de chon toan bo
            let has_pick = find_and_click_with_timeout("buttons/auto_pick.png", "Nut chon nhanh auto_pick", false)?;
            if !has_pick {
                add_log("Khong thay nut auto_pick.png, ket thuc vong lap ban.".to_string());
                break;
            }

            let now_click1 = Instant::now();
            add_log("[BAN DETAILED] Bat dau click ban lan 1...".to_string());
            find_and_click_with_timeout("buttons/sell-produce.png", "Nut bam ban lan 1", true)?;
            add_log(format!("[BAN DETAILED] Da xong click ban lan 1 (Mat {}ms). Cho click lan 2...", now_click1.elapsed().as_millis()));

            let now_click2 = Instant::now();
            find_and_click_with_timeout("buttons/sell-produce.png", "Nut bam ban lan 2 xac nhan", true)?;
            add_log(format!("[BAN DETAILED] Da xong click ban lan 2 (Mat {}ms). Bat dau vao vong lap quet confirm/ok...", now_click2.elapsed().as_millis()));

            // Cho doi mot trong hai truong hop: hoac co confirm.png, hoac co ok.png truc tiep (lv thap) trong 1 giay (quet lien tuc)
            let mut resolved = false;
            let start_resolve = Instant::now();
            let resolve_timeout = Duration::from_secs(1);
            let mut loop_count = 0;

            while start_resolve.elapsed() < resolve_timeout {
                loop_count += 1;
                let elapsed_ms = start_resolve.elapsed().as_millis();
                
                let (aw, ah, screen_rgba) = capture_helper(&g, bind_hwnd, state)
                    .map_err(|e| format!("Loi chup check shop confirm: {}", e))?;

                // Kich ban A: Nong san cap cao, hien confirm.png de xac nhan
                if let Some((_, fx, fy, tw, th, _)) = find_template_with_variants(
                    &screen_rgba, &templates, "buttons/confirm.png", threshold, None
                ) {
                    let tx = (fx as f64 + tw as f64 / 2.0) as i32;
                    let ty = (fy as f64 + th as f64 / 2.0) as i32;
                    click_ld(bind_hwnd, aw, ah, tx, ty);
                    add_log(format!("[BAN DETAILED] [KICH BAN A] Thay confirm.png o frame {} (sau {}ms). Da click tai ({}, {}), doi {}ms...", loop_count, elapsed_ms, tx, ty, click_delay.as_millis()));
                    sleep!(click_delay);

                    // Sau do hien modal thanh cong, cho de an ok.png
                    let start_ok = Instant::now();
                    add_log("[BAN DETAILED] Dang quet tim nut ok.png sau khi confirm...".to_string());
                    find_and_click_with_timeout("buttons/ok.png", "Nut dong thanh cong ok", true)?;
                    add_log(format!("[BAN DETAILED] Da click ok.png (Mat {}ms)", start_ok.elapsed().as_millis()));
                    resolved = true;
                    break;
                }

                // Kich ban B: Nong san cap thap, chi hien duy nhat ok.png de dong truc tiep
                if let Some((_, fx, fy, tw, th, _)) = find_template_with_variants(
                    &screen_rgba, &templates, "buttons/ok.png", threshold, None
                ) {
                    let tx = (fx as f64 + tw as f64 / 2.0) as i32;
                    let ty = (fy as f64 + th as f64 / 2.0) as i32;
                    click_ld(bind_hwnd, aw, ah, tx, ty);
                    add_log(format!("[BAN DETAILED] [KICH BAN B] Thay ok.png truc tiep o frame {} (sau {}ms). Da click tai ({}, {}), doi {}ms...", loop_count, elapsed_ms, tx, ty, click_delay.as_millis()));
                    sleep!(click_delay);
                    resolved = true;
                    break;
                }

                sleep!(Duration::from_millis(10));
            }

            if !resolved {
                add_log("[BALO HẾT TRÁI] Khong thay confirm.png hoac ok.png sau khi an ban. Balo da het sach trai, thoat vong lap ban.".to_string());
                break;
            }

            sell_success_count += 1;
        }

        add_log(format!("Da hoan thanh {} luot ban nong san o chu ky nay.", sell_success_count));

        // Thoát hoàn toàn
        find_and_click_with_timeout("buttons/close-harvest.png", "Dong cua hang ban", true)?;
        // Đợi tối đa 1 giây để kiểm tra và click leave2.png nếu có, không bắt buộc tồn tại
        crate::automation::utils::find_and_click_with_timeout(
            state, &g, bind_hwnd, &templates, "buttons/leave2.png", "Thoat han ra ngoai",
            Duration::from_secs(1), threshold, click_delay, false, &add_log
        )?;

        // Nếu không thu hoạch được gì mới và cũng không bán được gì -> Dừng kịch bản ngay để tránh lặp vô hạn (chỉ còn trái khóa)
        if harvest_success_count == 0 && sell_success_count == 0 {
            add_log("Phat hien khong thu hoach duoc gi moi va cung khong ban duoc gi (Chi con toan trai khoa). Dung kich ban de tranh lap vo han!".to_string());
            break;
        }

        // Nếu lúc nãy bot xác định là đã HẾT QUẢ trên cây -> dừng hoàn toàn kịch bản
        if !has_more_fruit {
            add_log("Da het sach qua tren cay va da ban het. Hoan thanh kich ban!".to_string());
            break;
        }

        add_log("Van con qua tren cay (do truoc do phat hien single-harvest.png). Lap lai chu ky Thu hoach & Ban tiep theo...".to_string());
    }

    add_log("===== KICH BAN THU HOACH & BAN HOAN THANH MY MAN =====".to_string());

    let result_log = logs.borrow().join("\n");
    Ok(result_log)
}
