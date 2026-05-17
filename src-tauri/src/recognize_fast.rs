pub struct FastRecognizer;

impl FastRecognizer {
    pub fn find_template_step(
        img_rgb: &[u8], img_w: usize, img_h: usize, img_stride: usize,
        tpl_rgb: &[u8], tpl_w: usize, tpl_h: usize,
        threshold: u32
    ) -> Option<(usize, usize, u32)> {
        if tpl_w > img_w || tpl_h > img_h { return None; }

        // Tăng threshold lên một chút cho các trường hợp biên
        let adjusted_threshold = threshold;

        for y in 0..img_h - tpl_h {
            for x in 0..img_w - tpl_w {
                // 1. Kiểm tra nhanh (Quick Check) tại 5 điểm vàng: 4 góc và tâm
                // Đây là bước lọc cực nhanh để loại bỏ 99% vùng không khớp
                let mut quick_possible = true;
                let test_points = [
                    (0, 0), (tpl_w - 1, 0), 
                    (tpl_w / 2, tpl_h / 2), 
                    (0, tpl_h - 1), (tpl_w - 1, tpl_h - 1)
                ];

                for (tx, ty) in test_points {
                    let img_idx = ((y + ty) * img_w + (x + tx)) * img_stride;
                    let tpl_idx = (ty * tpl_w + tx) * 3;
                    
                    // Tính cường độ sáng (Luminance) để nhạy hơn với ảnh đen trắng
                    let img_gray = (img_rgb[img_idx] as i32 + img_rgb[img_idx+1] as i32 + img_rgb[img_idx+2] as i32) / 3;
                    let tpl_gray = (tpl_rgb[tpl_idx] as i32 + tpl_rgb[tpl_idx+1] as i32 + tpl_rgb[tpl_idx+2] as i32) / 3;
                    
                    if (img_gray - tpl_gray).abs() as u32 > adjusted_threshold * 2 {
                        quick_possible = false;
                        break;
                    }
                }

                if quick_possible {
                    let mut total_sad: u64 = 0;
                    // Quét nhảy cách pixel (step 2) để tăng tốc độ cho ảnh lớn, 
                    // nhưng vẫn đảm bảo độ chính xác nhờ thuật toán tính trung bình.
                    let skip = if tpl_w * tpl_h > 5000 { 2 } else { 1 };
                    let mut count = 0;

                    for ty in (0..tpl_h).step_by(skip) {
                        for tx in (0..tpl_w).step_by(skip) {
                            let img_idx = ((y + ty) * img_w + (x + tx)) * img_stride;
                            let tpl_idx = (ty * tpl_w + tx) * 3;

                            // Thuật toán so sánh Hybrid: Ưu tiên sự chênh lệch tổng thể thay vì từng kênh lẻ
                            let r_diff = (img_rgb[img_idx] as i32 - tpl_rgb[tpl_idx] as i32).abs();
                            let g_diff = (img_rgb[img_idx+1] as i32 - tpl_rgb[tpl_idx+1] as i32).abs();
                            let b_diff = (img_rgb[img_idx+2] as i32 - tpl_rgb[tpl_idx+2] as i32).abs();
                            
                            // Đối với ảnh đen trắng, r_diff, g_diff, b_diff sẽ gần như bằng nhau.
                            // Việc cộng trung bình giúp khử nhiễu từ giả lập cực tốt.
                            let diff = (r_diff + g_diff + b_diff) / 3;
                            total_sad += diff as u64;
                            count += 1;
                        }
                    }
                    
                    let score = (total_sad / count.max(1)) as u32;
                    if score < adjusted_threshold {
                        return Some((x, y, score));
                    }
                }
            }
        }
        None
    }
}
