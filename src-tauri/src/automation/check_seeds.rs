use std::collections::HashMap;
use std::fs;
use std::sync::Arc;
use std::time::Duration;
use tauri::State;
use image::GenericImageView;

use crate::config::state::{AppState, CachedTemplate};
use crate::config::persisted::AppConfig;
use crate::core::recognize::FastRecognizer;
use crate::commands::get_or_create_grabber;
use crate::automation::utils::{
    check_and_clear_cancelled, swipe_ld, click_ld, capture_helper, BASE_W, BASE_H
};

pub fn run_check_seeds_templates_logic(state: &State<'_, AppState>) -> std::result::Result<String, String> {
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

    add_log(format!("===== BAT DAU CHECK LOI ANH MAU HAT GIONG ({}) =====", serial));

    // Nap toan bo templates
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

    // Kiem tra xem Shop da mo san chua
    let (_, _, screen_rgba) = capture_helper(&g, bind_hwnd, state)?;
    let gift_key = "buttons/gift.png";
    let shop_opened = if let Some((template_data, tw, th)) = templates.get(gift_key) {
        FastRecognizer::find_template_step(
            &screen_rgba, BASE_W as usize, BASE_H as usize, 4,
            template_data, *tw as usize, *th as usize, threshold, None
        ).is_some()
    } else {
        false
    };

    if !shop_opened {
        add_log("[CANH BAO] Shop chua mo san! Dang tu dong mo shop...".to_string());
        let start_time = std::time::Instant::now();
        let mut shop_opened = false;
        let mut step_b_reached = false;

        let seed_shop_key = "buttons/seed-shop.png";
        let open_shop_key = "buttons/open-seed-shop.png";
        let click_delay = Duration::from_millis(if config.click_delay_ms == 0 { 1000 } else { config.click_delay_ms });

        while start_time.elapsed() < shop_timeout_duration {
            if check_and_clear_cancelled(state) {
                add_log("[DUNG] Da dung kich ban do nguoi dung yeu cau.".to_string());
                return Err(logs.borrow().join("\n"));
            }
            let (aw, ah, screen_rgba) = capture_helper(&g, bind_hwnd, state)?;

            // TRANG THAI D: Shop da mo san (thay gift.png)
            if let Some((template_data, tw, th)) = templates.get(gift_key) {
                if FastRecognizer::find_template_step(
                    &screen_rgba, BASE_W as usize, BASE_H as usize, 4,
                    template_data, *tw as usize, *th as usize, threshold, None
                ).is_some() {
                     add_log("[MO SHOP D] Shop da mo thanh cong!".to_string());
                     shop_opened = true;
                     break;
                }
            }

            // TRANG THAI B: Thay nut open-seed-shop.png, click no
            let opt_shop1 = if let Some((template_data, tw, th)) = templates.get(open_shop_key) {
                FastRecognizer::find_template_step(
                    &screen_rgba, BASE_W as usize, BASE_H as usize, 4,
                    template_data, *tw as usize, *th as usize, threshold, None
                )
            } else {
                None
            };

            if let Some((fx, fy, score)) = opt_shop1 {
                if !step_b_reached {
                    add_log(format!("[MO SHOP B] Da tiep can NPC! Phat hien open-seed-shop.png (Score: {})!", score));
                    step_b_reached = true;
                }

                let tx = (fx as f64 + templates.get(open_shop_key).unwrap().1 as f64 / 2.0) as i32;
                let ty = (fy as f64 + templates.get(open_shop_key).unwrap().2 as f64 / 2.0) as i32;

                add_log(format!("[MO SHOP B] Click open-seed-shop.png tai ({}, {}) | Bo qua doi thoai NPC bang cach click 3 lan moi lan cach nhau 300ms...", tx, ty));
                click_ld(bind_hwnd, aw, ah, tx, ty);
                for _ in 0..2 {
                    std::thread::sleep(Duration::from_millis(300));
                    click_ld(bind_hwnd, aw, ah, tx, ty);
                }
                sleep!(click_delay);
                continue;
            }

            // TRANG THAI A: Chua co gi, click buttons/seed-shop.png de di chuyen toi NPC
            let opt_seed_shop = if let Some((template_data, tw, th)) = templates.get(seed_shop_key) {
                FastRecognizer::find_template_step(
                    &screen_rgba, BASE_W as usize, BASE_H as usize, 4,
                    template_data, *tw as usize, *th as usize, threshold, None
                )
            } else {
                None
            };

            if let Some((fx, fy, score)) = opt_seed_shop {
                let tx = (fx as f64 + templates.get(seed_shop_key).unwrap().1 as f64 / 2.0) as i32;
                let ty = (fy as f64 + templates.get(seed_shop_key).unwrap().2 as f64 / 2.0) as i32;

                add_log(format!("[MO SHOP A] Click buttons/seed-shop.png tai ({}, {}) | Score: {} | Doi {}ms...", tx, ty, score, click_delay.as_millis()));
                click_ld(bind_hwnd, aw, ah, tx, ty);
                sleep!(click_delay);
            } else {
                sleep!(Duration::from_millis(300));
            }
        }

        if !shop_opened {
            add_log("[THAT BAI] Qua thoi gian Timeout nhung khong mo duoc shop.".to_string());
            return Err(logs.borrow().join("\n"));
        }
    } else {
        add_log("[SHOP DA MO SAN] Bat dau quet chan doan hat giong...".to_string());
    }

    // 1. Quet tim tat ca cac file anh hat giong trong templates/seeds/
    let mut all_seeds_to_check = Vec::new();
    if let Ok(entries) = fs::read_dir("templates/seeds") {
        for entry in entries.flatten() {
            let path = entry.path();
            if let Some(ext) = path.extension() {
                if ext == "png" {
                    let file_name = path.file_name().unwrap().to_string_lossy().into_owned();
                    all_seeds_to_check.push(format!("seeds/{}", file_name));
                }
            }
        }
    }
    all_seeds_to_check.sort();

    // Map luu ket qua kiem tra: seed_name -> (tx, ty, score)
    let mut found_results: HashMap<String, (i32, i32, u32)> = HashMap::new();

    // 2. Chay vong lap cuon giong het ham buy_seeds (check diem dau va diem cuoi)
    let mut scrolling_down = true;
    for loop_count in 1..=30 {
        if check_and_clear_cancelled(state) {
            add_log("[DUNG] Da dung kich ban do nguoi dung yeu cau.".to_string());
            return Err(logs.borrow().join("\n"));
        }
        add_log(format!("--- Vong quet thu {} (Huong: {}) ---", loop_count, if scrolling_down { "Xuong" } else { "Lên" }));

        let (aw, ah, screen_rgba) = capture_helper(&g, bind_hwnd, state)?;

        // Quet so khop tat ca cac hat
        for target in &all_seeds_to_check {
            if check_and_clear_cancelled(state) {
                add_log("[DUNG] Da dung kich ban do nguoi dung yeu cau.".to_string());
                return Err(logs.borrow().join("\n"));
            }
            if let Some((template_data, tw, th)) = templates.get(target) {
                let max_scan_w = Some((BASE_W / 2) as usize);

                if let Some((fx, fy, score)) = FastRecognizer::find_template_step(
                    &screen_rgba, BASE_W as usize, BASE_H as usize, 4,
                    template_data, *tw as usize, *th as usize, 12, max_scan_w
                ) {
                    let tx = (fx as f64 + *tw as f64 / 2.0) as i32;
                    let ty = (fy as f64 + *th as f64 / 2.0) as i32;

                    let entry = found_results.entry(target.clone()).or_insert((tx, ty, score));
                    if score < entry.2 {
                        *entry = (tx, ty, score);
                    }
                }
            }
        }

        // Kiem tra diem dau va diem cuoi
        let mut found_jujube = false;
        let jujube_key = "seeds/jujube.png";
        if let Some((template_data, tw, th)) = templates.get(jujube_key) {
            let max_scan_w = Some((BASE_W / 2) as usize);
            if let Some((_, _, _)) = FastRecognizer::find_template_step(
                &screen_rgba, BASE_W as usize, BASE_H as usize, 4,
                template_data, *tw as usize, *th as usize, 12, max_scan_w
            ) {
                found_jujube = true;
            }
        }

        let mut found_carrot = false;
        let carrot_key = "seeds/carrot.png";
        if let Some((template_data, tw, th)) = templates.get(carrot_key) {
            let max_scan_w = Some((BASE_W / 2) as usize);
            if let Some((_, _, _)) = FastRecognizer::find_template_step(
                &screen_rgba, BASE_W as usize, BASE_H as usize, 4,
                template_data, *tw as usize, *th as usize, 12, max_scan_w
            ) {
                found_carrot = true;
            }
        }

        // Neu dang cuon len ma gap lai dau trang (carrot.png) thi coi nhu hoan tat chan doan day du!
        if !scrolling_down && found_carrot {
            add_log("[HOAN TAT] Da chan doan xong toan bo danh sach va cuon tro lai dau trang.".to_string());
            break;
        }

        // Thuc hien cuon giong het ham buy_seeds
        if scrolling_down {
            if found_jujube {
                add_log("[DAO CHIEU] Gap cuoi danh sach (jujube.png). Chuyen sang cuon LEN...".to_string());
                scrolling_down = false;
                swipe_ld(bind_hwnd, aw, ah, 240, 150, 240, 450);
            } else {
                add_log("Cuon xuong duoi...".to_string());
                swipe_ld(bind_hwnd, aw, ah, 240, 400, 240, 150);
            }
        } else {
            add_log("Cuon nguoc len tren...".to_string());
            swipe_ld(bind_hwnd, aw, ah, 240, 150, 240, 450);
        }

        sleep!(Duration::from_millis(600));
    }

    // 4. Tong hop bao cao
    add_log("===== BAO CAO CHAN DOAN HAT GIONG =====".to_string());
    let mut ok_count = 0;
    for target in &all_seeds_to_check {
        if let Some((tx, ty, score)) = found_results.get(target) {
            add_log(format!("[OK] {} -> Toa do: ({}, {}) | Score: {}", target, tx, ty, score));
            ok_count += 1;
        } else {
            add_log(format!("[LOI] {} -> Khong tim thay tren man hinh!", target));
        }
    }

    add_log(format!("===== HOAN THANH: Tim thay {}/{} anh mau hat giong =====", ok_count, all_seeds_to_check.len()));
    Ok(logs.into_inner().join("\n"))
}
