pub struct FastRecognizer;

impl FastRecognizer {
    pub fn find_template_step(
        img_rgb: &[u8], img_w: usize, img_h: usize,
        tpl_rgb: &[u8], tpl_w: usize, tpl_h: usize,
        threshold: u32
    ) -> Option<(usize, usize, u32)> {
        if tpl_w > img_w || tpl_h > img_h { return None; }

        // Quét từng pixel để đạt độ chính xác tuyệt đối
        for y in 0..img_h - tpl_h {
            for x in 0..img_w - tpl_w {
                let mut sad: u64 = 0;
                
                // Quét nhanh một số điểm đặc trưng thay vì chỉ 4 góc
                // Kiểm tra hàng giữa và cột giữa của template
                let mid_y = tpl_h / 2;
                let mut possible = true;
                for tx in (0..tpl_w).step_by(tpl_w / 4 + 1) {
                    let img_idx = ((y + mid_y) * img_w + (x + tx)) * 3;
                    let tpl_idx = (mid_y * tpl_w + tx) * 3;
                    let diff = (img_rgb[img_idx] as i32 - tpl_rgb[tpl_idx] as i32).abs() +
                               (img_rgb[img_idx+1] as i32 - tpl_rgb[tpl_idx+1] as i32).abs() +
                               (img_rgb[img_idx+2] as i32 - tpl_rgb[tpl_idx+2] as i32).abs();
                    if diff as u32 > threshold * 2 {
                        possible = false;
                        break;
                    }
                }

                if possible {
                    sad = 0;
                    // Quét toàn bộ template (hoặc nhảy cách 1 pixel nếu ảnh quá to)
                    let skip = if tpl_w * tpl_h > 10000 { 2 } else { 1 };
                    for ty in (0..tpl_h).step_by(skip) {
                        for tx in (0..tpl_w).step_by(skip) {
                            let img_idx = ((y + ty) * img_w + (x + tx)) * 3;
                            let tpl_idx = (ty * tpl_w + tx) * 3;
                            let diff = (img_rgb[img_idx] as i32 - tpl_rgb[tpl_idx] as i32).abs() +
                                       (img_rgb[img_idx+1] as i32 - tpl_rgb[tpl_idx+1] as i32).abs() +
                                       (img_rgb[img_idx+2] as i32 - tpl_rgb[tpl_idx+2] as i32).abs();
                            sad += diff as u64;
                        }
                    }
                    
                    let num_pixels = ((tpl_w / skip) * (tpl_h / skip)) as u64;
                    let score = (sad / num_pixels.max(1)) as u32;
                    if score < threshold {
                        return Some((x, y, score));
                    }
                }
            }
        }
        None
    }
}
