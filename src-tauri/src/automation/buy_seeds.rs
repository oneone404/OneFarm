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

pub fn run_buy_seeds_script_logic(state: &State<'_, AppState>, target_seeds: Vec<String>) -> std::result::Result<String, String> {
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
    // Mở shop là chu trình phức tạp cần di chuyển và chuyển cảnh, dùng timeout tối thiểu 30 giây
    let shop_timeout_duration = Duration::from_secs(timeout_secs.max(30));

    // Nạp mức độ nhạy nhận dạng động từ file cấu hình (mặc định 25)
    let threshold = if config.match_threshold == 0 { 25 } else { config.match_threshold };

    add_log(format!("===== BAT DAU KICH BAN MUA HAT GIONG ({}) =====", serial));
    add_log(format!("Danh sach hat giong can mua: {:?}", target_seeds));

    // Thực hiện tự động kiểm tra và đăng nhập game (Auto-Login check nếu cấu hình được bật)
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

    // 3. Quy trình tự động mở cửa hàng hạt giống bằng Máy Trạng Thái Tự Sửa Lỗi (Self-healing State Machine)
    let gift_key = "buttons/gift.png";
    let seed_shop_key = "buttons/seed-shop.png";
    let open_shop_key = "buttons/open-seed-shop.png";
    let open_shop2_key = "buttons/open-seed-shop2.png";

    let click_delay = Duration::from_millis(if config.click_delay_ms == 0 { 1000 } else { config.click_delay_ms });

    let mut has_gift = false;
    if let Ok((_aw, _ah, screen_rgba)) = capture_helper(&g, bind_hwnd, state) {
        has_gift = find_template_with_variants(
            &screen_rgba, &templates, gift_key, threshold, None
        ).is_some();
    }

    if !has_gift {
        add_log(format!("[SHOP CHƯA MỞ] Không tìm thấy buttons/gift.png. Tiến hành tự động mở bằng Máy Trạng Thái (Timeout {}s, Delay {}ms, Nguong {})...", timeout_secs.max(30), click_delay.as_millis(), threshold));

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

            // TRẠNG THÁI D: Shop đã mở thành công (có gift.png)
            let is_gift_visible = find_template_with_variants(
                &screen_rgba, &templates, gift_key, threshold, None
            ).is_some();

            if is_gift_visible {
                add_log("[MỞ SHOP THÀNH CÔNG] Đã phát hiện buttons/gift.png! Shop đã mở hoàn toàn.".to_string());
                shop_opened = true;
                break;
            }

            // TRẠNG THÁI C: Nút mở shop thực sự (open-seed-shop2.png) xuất hiện
            if let Some((found_key, fx, fy, tw, th, score)) = find_template_with_variants(
                &screen_rgba, &templates, open_shop2_key, threshold, None
            ) {
                if !step_c_reached {
                    add_log(format!("[MỞ SHOP C] Đã phát hiện {} (Score: {})!", found_key, score));
                    step_c_reached = true;
                }

                let tx = (fx as f64 + tw as f64 / 2.0) as i32;
                let ty = (fy as f64 + th as f64 / 2.0) as i32;

                add_log(format!("[MỞ SHOP C] Click {} tại ({}, {}) | Đợi {}ms...", found_key, tx, ty, click_delay.as_millis()));
                click_ld(bind_hwnd, aw, ah, tx, ty);
                sleep!(click_delay);
                continue;
            }

            // TRẠNG THÁI B: Nút hội thoại NPC (open-seed-shop.png) xuất hiện
            if let Some((found_key, fx, fy, tw, th, score)) = find_template_with_variants(
                &screen_rgba, &templates, open_shop_key, threshold, None
            ) {
                if !step_b_reached {
                    add_log(format!("[MỞ SHOP B] Đã tiếp cận NPC! Phát hiện {} (Score: {})!", found_key, score));
                    step_b_reached = true;
                }

                let tx = (fx as f64 + tw as f64 / 2.0) as i32;
                let ty = (fy as f64 + th as f64 / 2.0) as i32;

                add_log(format!("[MỞ SHOP B] Click {} tại ({}, {}) | Bỏ qua đối thoại NPC bằng cách click 3 lần cách nhau 300ms...", found_key, tx, ty));
                click_ld(bind_hwnd, aw, ah, tx, ty);
                for _ in 0..2 {
                    std::thread::sleep(Duration::from_millis(300));
                    click_ld(bind_hwnd, aw, ah, tx, ty);
                }
                sleep!(click_delay);
                continue;
            }

            // TRẠNG THÁI A: Chưa có gì, cần click buttons/seed-shop.png để di chuyển tới NPC
            if let Some((found_key, fx, fy, tw, th, score)) = find_template_with_variants(
                &screen_rgba, &templates, seed_shop_key, threshold, None
            ) {
                let tx = (fx as f64 + tw as f64 / 2.0) as i32;
                let ty = (fy as f64 + th as f64 / 2.0) as i32;

                add_log(format!("[MỞ SHOP A] Click {} tại ({}, {}) | Score: {} | Đợi {}ms...", found_key, tx, ty, score, click_delay.as_millis()));
                click_ld(bind_hwnd, aw, ah, tx, ty);
                sleep!(click_delay);
            } else {
                // Không tìm thấy gì, đợi một khoảng ngắn rồi thử lại
                sleep!(Duration::from_millis(300));
            }
        }

        if !shop_opened {
            add_log("[THẤT BẠI] Quá thời gian Timeout nhưng không mở được shop (thiếu buttons/gift.png).".to_string());
            return Err(logs.borrow().join("\n"));
        }
    } else {
        add_log("[SHOP ĐÃ MỞ SẴN] Đã tìm thấy buttons/gift.png. Bắt đầu quét mua hạt giống...".to_string());
    }

    let mut remaining_targets = target_seeds.clone();
    let mut scrolling_down = true;

    for loop_count in 1..=30 {
        if check_and_clear_cancelled(state) {
            add_log("[DUNG] Da dung kich ban do nguoi dung yeu cau.".to_string());
            return Err(logs.borrow().join("\n"));
        }
        if remaining_targets.is_empty() {
            add_log("Da mua het toan bo cac hat giong trong danh sach!".to_string());
            break;
        }

        add_log(format!("--- Vong quet thu {} (Huong: {}) ---", loop_count, if scrolling_down { "Xuong" } else { "Len" }));

        let (aw, ah, screen_rgba) = capture_helper(&g, bind_hwnd, state)?;

        let mut bought_in_this_step = Vec::new();
        for target in &remaining_targets {
            if true {
                let max_scan_w = Some((BASE_W / 2) as usize);

                if let Some((_found_key, fx, fy, tw, th, _)) = find_template_with_variants(
                    &screen_rgba, &templates, target, 12, max_scan_w
                ) {
                    let tx = (fx as f64 + tw as f64 / 2.0) as i32;
                    let ty = (fy as f64 + th as f64 / 2.0) as i32;

                    add_log(format!("[TIM THAY] Qua {} tai ({}, {}) | Click de chon...", target, tx, ty));
                    click_ld(bind_hwnd, aw, ah, tx, ty);

                    let sell_produce_key = "buttons/sell-produce.png";
                    let mut found_sell = false;
                    let mut sell_x = 0;
                    let mut sell_y = 0;
                    let mut aw_modal = aw;
                    let mut ah_modal = ah;

                    let start_check = std::time::Instant::now();
                    let timeout_ms = 500;

                    if true {
                        while start_check.elapsed().as_millis() < timeout_ms {
                            if check_and_clear_cancelled(state) {
                                return Err(logs.borrow().join("\n"));
                            }
                            if let Ok((curr_aw, curr_ah, screen_modal_rgba)) = capture_helper(&g, bind_hwnd, state) {
                                aw_modal = curr_aw;
                                ah_modal = curr_ah;
                                if let Some((found_key, fx_s, fy_s, tw, th, score_s)) = find_template_with_variants(
                                    &screen_modal_rgba, &templates, sell_produce_key, threshold, None
                                ) {
                                    sell_x = (fx_s as f64 + tw as f64 / 2.0) as i32;
                                    sell_y = (fy_s as f64 + th as f64 / 2.0) as i32;
                                    add_log(format!("[CON HANG] Tim thay {} tai ({}, {}) | Score: {} | Sau {}ms", found_key, sell_x, sell_y, score_s, start_check.elapsed().as_millis()));
                                    found_sell = true;
                                    break;
                                }
                            }
                            std::thread::sleep(std::time::Duration::from_millis(10));
                        }
                    }

                    if !found_sell {
                        add_log("[HET HANG] Khong tim thay nut sell-produce.png. Bo qua.".to_string());
                        bought_in_this_step.push(target.clone());
                    } else {
                        click_ld(bind_hwnd, aw_modal, ah_modal, sell_x, sell_y);
                        // Chờ 1 giây để thanh slider và giao diện hiển thị đầy đủ
                        sleep!(Duration::from_millis(1000));

                        let (aw_slider, ah_slider, screen_slider_rgba) = capture_helper(&g, bind_hwnd, state)?;
                        let slider_key = "buttons/slider.png";

                        if let Some((found_key, fx_sl, fy_sl, tw, th, score_sl)) = find_template_with_variants(
                            &screen_slider_rgba, &templates, slider_key, threshold, None
                        ) {
                            let slider_x = (fx_sl as f64 + tw as f64 / 2.0) as i32;
                            let slider_y = (fy_sl as f64 + th as f64 / 2.0) as i32;
                            add_log(format!("[SLIDER] Tim thay {} tai ({}, {}) | Score: {}", found_key, slider_x, slider_y, score_sl));

                            if slider_x <= 480 {
                                // Slider is on the left (quantity not maxed yet). Let's click it to max out.
                                let dest_slider_x = BASE_W as i32 - slider_x + 5;
                                add_log(format!("[SLIDER LEFT] Click tai ({}, {}) de dat max", dest_slider_x, slider_y));
                                click_ld(bind_hwnd, aw_slider, ah_slider, dest_slider_x, slider_y);
                            } else {
                                // Slider is already on the right (1 seed stock case). Skip slider clicking entirely.
                                add_log("[SLIDER ALREADY RIGHT] Nao o san ben phai (1 hat). Bo qua buoc click keo slider.".to_string());
                            }

                            // Chờ tầm 500ms sau khi kéo max để giao diện ổn định và cập nhật số lượng
                            sleep!(Duration::from_millis(500));

                            // Chụp màn hình mới chứa thông tin mua hàng đã cập nhật số lượng
                            let (aw_confirm, ah_confirm, screen_confirm_rgba) = capture_helper(&g, bind_hwnd, state)?;

                            if let Some((found_key, fx_c, fy_c, tw_c, th_c, score_c)) = find_template_with_variants(
                                &screen_confirm_rgba, &templates, sell_produce_key, threshold, None
                            ) {
                                let conf_x = (fx_c as f64 + tw_c as f64 / 2.0) as i32;
                                let conf_y = (fy_c as f64 + th_c as f64 / 2.0) as i32;

                                let recognized_qty = crate::automation::utils::recognize_number_above_slider(
                                    &screen_confirm_rgba, &templates, slider_y as usize
                                ).unwrap_or(1);
                                add_log(format!("[XAC NHAN MUA] Click {} tai ({}, {}) | Nhan dang so luong: {} hat | Score: {}", found_key, conf_x, conf_y, recognized_qty, score_c));
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
        }

        remaining_targets.retain(|t| !bought_in_this_step.contains(t));

        if remaining_targets.is_empty() {
            add_log("Da mua het toan bo. Hoan thanh kich ban!".to_string());
            break;
        }

        let mut found_jujube = false;
        let jujube_key = "seeds/jujube.png";
        if true {
            let max_scan_w = Some((BASE_W / 2) as usize);
            if let Some((_, _, _, _, _, _)) = find_template_with_variants(
                &screen_rgba, &templates, jujube_key, 18, max_scan_w
            ) {
                found_jujube = true;
                add_log("[SCROLL] Da thay jujube.png (hoac bien the), san sang vuot man hinh...".to_string());
            }
        }
        let mut found_carrot = false;
        let carrot_key = "seeds/carrot.png";
        if true {
            let max_scan_w = Some((BASE_W / 2) as usize);
            if let Some((_, _, _, _, _, _)) = find_template_with_variants(
                &screen_rgba, &templates, carrot_key, 18, max_scan_w
            ) {
                found_carrot = true;
            }
        }

        if scrolling_down {
            if found_jujube {
                add_log("[ĐẢO CHIỀU] Gặp cuối danh sách (jujube.png). Chuyển sang cuộn LÊN...".to_string());
                scrolling_down = false;
                swipe_ld(bind_hwnd, aw, ah, 240, 150, 240, 450);
            } else {
                add_log("Cuon xuong duoi...".to_string());
                swipe_ld(bind_hwnd, aw, ah, 240, 400, 240, 150);
            }
        } else {
            if found_carrot {
                add_log("[ĐẢO CHIỀU] Gặp đầu danh sách (carrot.png). Chuyển sang cuộn XUỐNG...".to_string());
                scrolling_down = true;
                swipe_ld(bind_hwnd, aw, ah, 240, 400, 240, 150);
            } else {
                add_log("Cuon nguoc len tren...".to_string());
                swipe_ld(bind_hwnd, aw, ah, 240, 150, 240, 450);
            }
        }

        sleep!(Duration::from_millis(600));
    }

    // Save purchased quantities to in-memory purchase history inside AppState
    if !total_bought.is_empty() {
        let mut history = state.seed_purchase_history.lock().unwrap();
        for (seed, qty) in &total_bought {
            let entry = history.entry(seed.clone()).or_insert(0);
            *entry += qty;
        }
    }

    add_log("===== KẾT QUẢ MUA HẠT GIỐNG =====".to_string());
    if total_bought.is_empty() {
        add_log("Khong mua duoc hat giong nao.".to_string());
    } else {
        for (seed, qty) in &total_bought {
            let seed_name = seed.strip_prefix("seeds/").unwrap_or(seed).strip_suffix(".png").unwrap_or(seed);
            add_log(format!("- Mua thanh cong: {} x {} hạt", seed_name, qty));
        }
    }
    add_log("=================================".to_string());
    Ok(logs.into_inner().join("\n"))
}
