use std::sync::Arc;
use std::collections::HashMap;
use std::fs;
use std::time::Duration;
use tauri::State;
use image::GenericImageView;

use crate::config::state::{AppState, CachedTemplate};
use crate::config::persisted::AppConfig;
use crate::commands::get_or_create_grabber;
use crate::automation::utils::{
    check_and_clear_cancelled, swipe_ld, click_ld, capture_helper, find_template_with_variants, BASE_W
};

pub fn run_buy_tools_script_logic(state: &State<'_, AppState>, target_tools: Vec<String>) -> std::result::Result<String, String> {
    use std::cell::RefCell;
    let logs = RefCell::new(Vec::new());
    let add_log = |msg: String| {
        println!("{}", msg);
        logs.borrow_mut().push(msg);
    };

    macro_rules! sleep {
        ($dur:expr) => {
            let start = std::time::Instant::now();
            let step = std::time::Duration::from_millis(20);
            while start.elapsed() < $dur {
                if check_and_clear_cancelled(state) {
                    return Err(logs.borrow().join("\n"));
                }
                std::thread::sleep(step);
            }
        };
    }

    let (g, _, bind_hwnd, serial) = get_or_create_grabber(state)?;

    let config = AppConfig::load();
    let timeout_secs = if config.button_timeout_secs == 0 { 5 } else { config.button_timeout_secs };
    let shop_timeout_duration = Duration::from_secs(timeout_secs.max(30));
    let threshold = if config.match_threshold == 0 { 25 } else { config.match_threshold };

    add_log(format!("===== BAT DAU KICH BAN MUA CONG CU ({}) =====", serial));
    add_log(format!("Danh sach cong cu can mua: {:?}", target_tools));

    if config.enable_auto_login {
        if let Err(e) = crate::automation::utils::ensure_game_ready(state, &add_log) {
            add_log(format!("[THAT BAI] Lỗi Auto-Login: {}", e));
            return Err(logs.borrow().join("\n"));
        }
    }

    let mut total_bought: HashMap<String, u32> = HashMap::new();

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

    let tool_shop_key = "buttons/tool-shop.png";
    let open_shop_key = "buttons/open-tool-shop.png";
    let open_shop2_key = "buttons/open-tool-shop2.png";
    let close_shop_key = "buttons/close-seed.png";

    let click_delay = Duration::from_millis(if config.click_delay_ms == 0 { 1000 } else { config.click_delay_ms });

    // ─── BƯỚC 0: KIỂM TRA VÀ TẮT MODAL MUA HẠT (NẾU ĐANG MỞ ĐÈ) ─────────────────────
    if let Ok((_aw, _ah, screen_rgba)) = capture_helper(&g, bind_hwnd, state) {
        let is_seed_modal_open = find_template_with_variants(
            &screen_rgba, &templates, "buttons/gift.png", threshold, None
        ).is_some();

        if is_seed_modal_open {
            add_log("Phat hien dang o modal mua hat. Tien hanh thoat truoc khi mo shop cong cu...".to_string());
            let find_and_click_with_timeout = |key: &str, name_vi: &str, must_exist: bool| -> std::result::Result<bool, String> {
                crate::automation::utils::find_and_click_with_timeout(
                    state, &g, bind_hwnd, &templates, key, name_vi,
                    Duration::from_secs(timeout_secs), threshold, click_delay, must_exist, &add_log
                )
            };
            let _ = find_and_click_with_timeout("buttons/close-seed.png", "Dong modal mua hat", true);
            let _ = find_and_click_with_timeout("buttons/leave.png", "Dong hop thoai thoat 1", true);
            let _ = crate::automation::utils::find_and_click_with_timeout(
                state, &g, bind_hwnd, &templates, "buttons/leave2.png", "Dong hop thoai thoat 2",
                Duration::from_secs(1), threshold, click_delay, false, &add_log
            );
        } else {
            add_log("Khong o modal mua hat, bo qua buoc dong phu.".to_string());
        }
    }

    let mut is_shop_already_open = false;
    if let Ok((_aw, _ah, screen_rgba)) = capture_helper(&g, bind_hwnd, state) {
        is_shop_already_open = find_template_with_variants(
            &screen_rgba, &templates, close_shop_key, threshold, None
        ).is_some();
    }

    if !is_shop_already_open {
        add_log(format!("[SHOP CHƯA MỞ] Chưa phát hiện close-seed.png. Tiến hành tự động mở shop công cụ (Timeout {}s)...", timeout_secs.max(30)));
        let start_time = std::time::Instant::now();
        let mut step_b_reached = false;
        let mut step_c_reached = false;
        let mut shop_opened = false;

        while start_time.elapsed() < shop_timeout_duration {
            if check_and_clear_cancelled(state) {
                add_log("[DUNG] Da dung kich ban do nguoi dung yeu cau.".to_string());
                return Err(logs.borrow().join("\n"));
            }
            let (aw, ah, screen_rgba) = capture_helper(&g, bind_hwnd, state)?;

            // TRẠNG THÁI D: Shop đã mở thành công (có close-seed.png)
            if find_template_with_variants(&screen_rgba, &templates, close_shop_key, threshold, None).is_some() {
                add_log("[MỞ SHOP THÀNH CÔNG] Đã phát hiện buttons/close-seed.png! Shop công cụ đã mở.".to_string());
                shop_opened = true;
                break;
            }

            // TRẠNG THÁI C: Nút mở shop thực sự (open-tool-shop2.png) xuất hiện
            if let Some((found_key, fx, fy, tw, th, score)) = find_template_with_variants(
                &screen_rgba, &templates, open_shop2_key, threshold, None
            ) {
                if !step_c_reached {
                    add_log(format!("[MỞ SHOP C] Đã phát hiện {} (Score: {})!", found_key, score));
                    step_c_reached = true;
                }
                let tx = (fx as f64 + tw as f64 / 2.0) as i32;
                let ty = (fy as f64 + th as f64 / 2.0) as i32;
                add_log(format!("[MỞ SHOP C] Click {} tại ({}, {})", found_key, tx, ty));
                click_ld(bind_hwnd, aw, ah, tx, ty);
                sleep!(click_delay);
                continue;
            }

            // TRẠNG THÁI B: Nút hội thoại NPC (open-tool-shop.png) xuất hiện
            if let Some((found_key, fx, fy, tw, th, score)) = find_template_with_variants(
                &screen_rgba, &templates, open_shop_key, threshold, None
            ) {
                if !step_b_reached {
                    add_log(format!("[MỞ SHOP B] Đã tiếp cận NPC! Phát hiện {} (Score: {})!", found_key, score));
                    step_b_reached = true;
                }
                let tx = (fx as f64 + tw as f64 / 2.0) as i32;
                let ty = (fy as f64 + th as f64 / 2.0) as i32;
                add_log(format!("[MỞ SHOP B] Click {} tại ({}, {}) | Bỏ qua đối thoại NPC bằng cách click 3 lần...", found_key, tx, ty));
                click_ld(bind_hwnd, aw, ah, tx, ty);
                for _ in 0..2 {
                    std::thread::sleep(Duration::from_millis(300));
                    click_ld(bind_hwnd, aw, ah, tx, ty);
                }
                sleep!(click_delay);
                continue;
            }

            // TRẠNG THÁI A: Chưa có gì, click buttons/tool-shop.png để di chuyển tới NPC
            if let Some((found_key, fx, fy, tw, th, score)) = find_template_with_variants(
                &screen_rgba, &templates, tool_shop_key, threshold, None
            ) {
                let tx = (fx as f64 + tw as f64 / 2.0) as i32;
                let ty = (fy as f64 + th as f64 / 2.0) as i32;
                add_log(format!("[MỞ SHOP A] Click {} tại ({}, {}) | Score: {}", found_key, tx, ty, score));
                click_ld(bind_hwnd, aw, ah, tx, ty);
                sleep!(click_delay);
            } else {
                sleep!(Duration::from_millis(300));
            }
        }

        if !shop_opened {
            add_log("[THẤT BẠI] Quá thời gian Timeout nhưng không mở được shop công cụ.".to_string());
            return Err(logs.borrow().join("\n"));
        }
    } else {
        add_log("[SHOP ĐÃ MỞ SẴN] Đã tìm thấy close-seed.png. Bắt đầu quét mua công cụ...".to_string());
    }

    let mut remaining_targets = target_tools.clone();
    let mut scrolling_down = true;

    for loop_count in 1..=10 {
        if check_and_clear_cancelled(state) {
            add_log("[DUNG] Da dung kich ban do nguoi dung yeu cau.".to_string());
            return Err(logs.borrow().join("\n"));
        }
        if remaining_targets.is_empty() {
            add_log("Da mua het toan bo cac cong cu trong danh sach!".to_string());
            break;
        }

        add_log(format!("--- Vong quet cong cu thu {} (Huong: {}) ---", loop_count, if scrolling_down { "Xuong" } else { "Len" }));

        let (aw, ah, screen_rgba) = capture_helper(&g, bind_hwnd, state)?;
        let mut bought_in_this_step = Vec::new();

        for target in &remaining_targets {
            let max_scan_w = Some((BASE_W / 2) as usize);
            if let Some((_found_key, fx, fy, tw, th, _)) = find_template_with_variants(
                &screen_rgba, &templates, target, 12, max_scan_w
            ) {
                let tx = (fx as f64 + tw as f64 / 2.0) as i32;
                let ty = (fy as f64 + th as f64 / 2.0) as i32;

                add_log(format!("[TIM THAY] Cong cu {} tai ({}, {}) | Click de chon...", target, tx, ty));
                click_ld(bind_hwnd, aw, ah, tx, ty);

                let sell_produce_key = "buttons/sell-produce.png";
                let mut found_sell = false;
                let mut sell_x = 0;
                let mut sell_y = 0;
                let mut aw_modal = aw;
                let mut ah_modal = ah;

                let start_check = std::time::Instant::now();
                let timeout_ms = 500;

                while start_check.elapsed().as_millis() < timeout_ms {
                    if check_and_clear_cancelled(state) {
                        return Err(logs.borrow().join("\n"));
                    }
                    if let Ok((curr_aw, curr_ah, screen_modal_rgba)) = capture_helper(&g, bind_hwnd, state) {
                        aw_modal = curr_aw;
                        ah_modal = curr_ah;
                        if let Some((found_key, fx_s, fy_s, tw_s, th_s, score_s)) = find_template_with_variants(
                            &screen_modal_rgba, &templates, sell_produce_key, threshold, None
                        ) {
                            sell_x = (fx_s as f64 + tw_s as f64 / 2.0) as i32;
                            sell_y = (fy_s as f64 + th_s as f64 / 2.0) as i32;
                            add_log(format!("[CON HANG] Tim thay {} tai ({}, {}) | Score: {}", found_key, sell_x, sell_y, score_s));
                            found_sell = true;
                            break;
                        }
                    }
                    std::thread::sleep(std::time::Duration::from_millis(10));
                }

                if !found_sell {
                    add_log("[HET HANG] Khong tim thay nut sell-produce.png. Bo qua.".to_string());
                    bought_in_this_step.push(target.clone());
                } else {
                    click_ld(bind_hwnd, aw_modal, ah_modal, sell_x, sell_y);
                    sleep!(Duration::from_millis(1000));

                    let (aw_slider, ah_slider, screen_slider_rgba) = capture_helper(&g, bind_hwnd, state)?;
                    let slider_key = "buttons/slider.png";

                    if let Some((found_key, fx_sl, fy_sl, tw_sl, th_sl, score_sl)) = find_template_with_variants(
                        &screen_slider_rgba, &templates, slider_key, threshold, None
                    ) {
                        let slider_x = (fx_sl as f64 + tw_sl as f64 / 2.0) as i32;
                        let slider_y = (fy_sl as f64 + th_sl as f64 / 2.0) as i32;
                        add_log(format!("[SLIDER] Tim thay {} tai ({}, {}) | Score: {}", found_key, slider_x, slider_y, score_sl));

                        if slider_x <= 480 {
                            let dest_slider_x = BASE_W as i32 - slider_x + 5;
                            add_log(format!("[SLIDER LEFT] Click tai ({}, {}) de dat max", dest_slider_x, slider_y));
                            click_ld(bind_hwnd, aw_slider, ah_slider, dest_slider_x, slider_y);
                        } else {
                            add_log("[SLIDER ALREADY RIGHT] Nao o san ben phai (1 mon). Bo qua click keo slider.".to_string());
                        }

                        sleep!(Duration::from_millis(500));

                        let (aw_confirm, ah_confirm, screen_confirm_rgba) = capture_helper(&g, bind_hwnd, state)?;
                        if let Some((found_key, fx_c, fy_c, tw_c, th_c, score_c)) = find_template_with_variants(
                            &screen_confirm_rgba, &templates, sell_produce_key, threshold, None
                        ) {
                            let conf_x = (fx_c as f64 + tw_c as f64 / 2.0) as i32;
                            let conf_y = (fy_c as f64 + th_c as f64 / 2.0) as i32;

                            let recognized_qty = crate::automation::utils::recognize_number_above_slider(
                                &screen_confirm_rgba, &templates, slider_y as usize
                            ).unwrap_or(1);
                            add_log(format!("[XAC NHAN MUA] Click {} tai ({}, {}) | Nhan dang so luong: {} mon | Score: {}", found_key, conf_x, conf_y, recognized_qty, score_c));
                            click_ld(bind_hwnd, aw_confirm, ah_confirm, conf_x, conf_y);

                            *total_bought.entry(target.clone()).or_insert(0) += recognized_qty;
                            bought_in_this_step.push(target.clone());
                            sleep!(Duration::from_millis(500));
                        }
                    } else {
                        add_log("[SLIDER NOT FOUND] Khong thay slider.png. Bo qua.".to_string());
                    }
                }
            }
        }

        remaining_targets.retain(|t| !bought_in_this_step.contains(t));

        if remaining_targets.is_empty() {
            add_log("Da mua het toan bo. Hoan thanh!".to_string());
            break;
        }

        // Với công cụ thường danh sách rất ngắn, ta chỉ cần cuộn nhẹ xuống/lên một lần để tìm kiếm thêm
        if scrolling_down {
            add_log("Cuon xuong duoi...".to_string());
            swipe_ld(bind_hwnd, aw, ah, 240, 400, 240, 150);
            scrolling_down = false;
        } else {
            add_log("Cuon nguoc len tren...".to_string());
            swipe_ld(bind_hwnd, aw, ah, 240, 150, 240, 450);
            scrolling_down = true;
        }
        sleep!(Duration::from_millis(600));
    }

    // Nghỉ 1 giây để giao diện ổn định sau giao dịch cuối cùng trước khi đóng shop
    sleep!(Duration::from_secs(1));

    // Đóng cửa hàng công cụ
    add_log("Tien hanh dong cua hang cong cu bang cach click buttons/close-seed.png...".to_string());
    let mut closed_successfully = false;
    let start_close = std::time::Instant::now();
    while start_close.elapsed() < shop_timeout_duration {
        if check_and_clear_cancelled(state) {
            return Err(logs.borrow().join("\n"));
        }
        let (aw, ah, screen_rgba) = capture_helper(&g, bind_hwnd, state)?;
        if let Some((found_key, fx, fy, tw, th, score)) = find_template_with_variants(
            &screen_rgba, &templates, close_shop_key, threshold, None
        ) {
            let tx = (fx as f64 + tw as f64 / 2.0) as i32;
            let ty = (fy as f64 + th as f64 / 2.0) as i32;
            add_log(format!("[ĐÓNG SHOP] Click {} tại ({}, {}) | Score: {} | Bỏ qua đối thoại NPC...", found_key, tx, ty, score));
            click_ld(bind_hwnd, aw, ah, tx, ty);
            for _ in 0..2 {
                std::thread::sleep(Duration::from_millis(300));
                click_ld(bind_hwnd, aw, ah, tx, ty);
            }
            sleep!(click_delay);
            closed_successfully = true;
            break;
        } else {
            // Nếu không thấy close-seed.png nữa chứng tỏ đã đóng thành công
            closed_successfully = true;
            break;
        }
    }

    if closed_successfully {
        add_log("[ĐÓNG SHOP] Đã đóng cửa hàng thành công.".to_string());
    }

    // Kiểm tra và click nút leave2.png trong 1 giây nếu xuất hiện
    let leave2_key = "buttons/leave2.png";
    add_log("Dang quet tim buttons/leave2.png trong 1 giay...".to_string());
    let start_leave2 = std::time::Instant::now();
    let leave2_timeout = Duration::from_secs(1);
    while start_leave2.elapsed() < leave2_timeout {
        if check_and_clear_cancelled(state) {
            return Err(logs.borrow().join("\n"));
        }
        if let Ok((aw, ah, screen_rgba)) = capture_helper(&g, bind_hwnd, state) {
            if let Some((found_key, fx, fy, tw, th, score)) = find_template_with_variants(
                &screen_rgba, &templates, leave2_key, threshold, None
            ) {
                let tx = (fx as f64 + tw as f64 / 2.0) as i32;
                let ty = (fy as f64 + th as f64 / 2.0) as i32;
                add_log(format!("[THOÁT NPC] Phát hiện {} tại ({}, {}) | Score: {} | Click thoat.", found_key, tx, ty, score));
                click_ld(bind_hwnd, aw, ah, tx, ty);
                sleep!(click_delay);
                break;
            }
        }
        std::thread::sleep(Duration::from_millis(100));
    }

    add_log("===== KẾT QUẢ MUA CÔNG CỤ =====".to_string());
    if total_bought.is_empty() {
        add_log("Khong mua duoc cong cu nao.".to_string());
    } else {
        for (tool, qty) in &total_bought {
            let tool_name = tool.strip_prefix("tools/").unwrap_or(tool).strip_suffix(".png").unwrap_or(tool);
            add_log(format!("- Mua thanh cong: {} x {} cái", tool_name, qty));
        }
    }
    add_log("================================".to_string());

    // Ghi số lượng công cụ đã mua vào lịch sử mua hàng chung trong AppState
    if !total_bought.is_empty() {
        let mut history = state.seed_purchase_history.lock().unwrap();
        for (tool, qty) in &total_bought {
            let entry = history.entry(tool.clone()).or_insert(0);
            *entry += qty;
        }
    }

    Ok(logs.into_inner().join("\n"))
}
