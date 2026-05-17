pub struct FastRecognizer;

impl FastRecognizer {
    pub fn find_template_step(
        img_rgb: &[u8], img_w: usize, img_h: usize, img_stride: usize,
        tpl_rgb: &[u8], tpl_w: usize, tpl_h: usize,
        threshold: u32, max_scan_w: Option<usize>
    ) -> Option<(usize, usize, u32)> {
        if tpl_w > img_w || tpl_h > img_h { return None; }

        let limit_w = max_scan_w.unwrap_or(img_w).min(img_w);
        if tpl_w > limit_w { return None; }

        let adjusted_threshold = threshold;

        for y in 0..img_h - tpl_h {
            for x in 0..limit_w - tpl_w {
                let mut quick_possible = true;
                let inset_x = (tpl_w / 6).max(1);
                let inset_y = (tpl_h / 6).max(1);
                let test_points = [
                    (inset_x, inset_y), 
                    (tpl_w - 1 - inset_x, inset_y), 
                    (inset_x, tpl_h - 1 - inset_y), 
                    (tpl_w - 1 - inset_x, tpl_h - 1 - inset_y),
                    (tpl_w / 2, tpl_h / 2),
                    (tpl_w / 3, tpl_h / 3),
                    (tpl_w * 2 / 3, tpl_h / 3),
                    (tpl_w / 3, tpl_h * 2 / 3),
                    (tpl_w * 2 / 3, tpl_h * 2 / 3),
                ];

                for (tx, ty) in test_points {
                    let img_idx = ((y + ty) * img_w + (x + tx)) * img_stride;
                    let tpl_idx = (ty * tpl_w + tx) * 3;
                    
                    let img_gray = (img_rgb[img_idx] as i32 + img_rgb[img_idx+1] as i32 + img_rgb[img_idx+2] as i32) / 3;
                    let tpl_gray = (tpl_rgb[tpl_idx] as i32 + tpl_rgb[tpl_idx+1] as i32 + tpl_rgb[tpl_idx+2] as i32) / 3;
                    
                    if (img_gray - tpl_gray).abs() as u32 > adjusted_threshold * 2 {
                        quick_possible = false;
                        break;
                    }
                }

                if quick_possible {
                    let mut total_sad: u64 = 0;
                    let skip = if tpl_w * tpl_h > 5000 { 2 } else { 1 };
                    let mut count = 0;

                    for ty in (0..tpl_h).step_by(skip) {
                        for tx in (0..tpl_w).step_by(skip) {
                            let img_idx = ((y + ty) * img_w + (x + tx)) * img_stride;
                            let tpl_idx = (ty * tpl_w + tx) * 3;

                            let r_diff = (img_rgb[img_idx] as i32 - tpl_rgb[tpl_idx] as i32).abs();
                            let g_diff = (img_rgb[img_idx+1] as i32 - tpl_rgb[tpl_idx+1] as i32).abs();
                            let b_diff = (img_rgb[img_idx+2] as i32 - tpl_rgb[tpl_idx+2] as i32).abs();
                            
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
