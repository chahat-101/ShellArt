use image::RgbImage;
use std::f32;
const WEIGHTS: [f32; 3] = [0.299, 0.587, 0.114];
pub struct BlockSample {
    pub lum: f32,
    pub r: u8,
    pub b: u8,
    pub g: u8,
}

pub fn block_color(img: &RgbImage, x0: u32, y0: u32, w: u32, h: u32) -> BlockSample {
    let (img_w, img_h) = img.dimensions();

    let mut r_sum = 0u32;
    let mut g_sum = 0u32;
    let mut b_sum = 0u32;
    let mut count = 0u32;

    let y_end = (y0 + h).min(img_h);
    let x_end = (x0 + w).min(img_w);

    for y in y0..y_end {
        for x in x0..x_end {
            let pixel = img.get_pixel(x, y);

            r_sum += pixel[0] as u32;
            g_sum += pixel[1] as u32;
            b_sum += pixel[2] as u32;
            count += 1;
        }
    }
    let r = (r_sum / count) as u8;
    let g = (g_sum / count) as u8;
    let b = (b_sum / count) as u8;

    let brightness = WEIGHTS[0] * r as f32 + WEIGHTS[1] * g as f32 + WEIGHTS[2] * b as f32;

    BlockSample {
        lum: brightness,
        r,
        g,
        b,
    }
}

pub fn calculate_block_size(
    img_width: u32,
    img_height: u32,
    target_width: u32,
    aspect: f32,
) -> (u32, u32) {
    let block_w = img_width as f32 / target_width as f32;
    let block_h = block_w as f32 / aspect;

    let block_w = block_w.max(1.0).round() as u32;
    let block_h = block_h.max(1.0).round() as u32;

    (block_w, block_h)
}
