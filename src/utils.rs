use clap::ValueEnum;
use opencv::core::Vec3b;
use opencv::videoio::VideoCapture;
use opencv::{Result, core, highgui, prelude::*, videoio};

#[derive(Default)]
pub struct BlockSample {
    pub lum: f32,
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

pub const WEIGHTS: [f32; 3] = [0.299, 0.587, 0.114];

pub fn get_frame_data(
    cam: &mut VideoCapture,
    frame: &mut Mat,
    flipped: bool,
    grey_scale: bool,
) -> Result<()> {
    cam.read(frame);

    if grey_scale {
        for row in 0..frame.rows() {
            for col in 0..frame.cols() {
                let mut pixel = frame.at_2d_mut::<Vec3b>(row, col)?;

                let grey_value = (pixel[0] as f32 * WEIGHTS[2]
                    + pixel[1] as f32 * WEIGHTS[1]
                    + pixel[2] as f32 * WEIGHTS[0]) as u8;

                pixel[0] = grey_value;
                pixel[1] = grey_value;
                pixel[2] = grey_value;
            }
        }
    }

    if flipped {
        let mut flipped_frame = Mat::default();
        core::flip(frame, &mut flipped_frame, 1);
        *frame = flipped_frame;
        return Ok(());
    }

    Ok(())
}

impl CharSet {
    pub fn get_chars(&self) -> &'static str {
        match self {
            CharSet::Retro => " ░▒▓█",
            CharSet::Default => "%@#*+=-:.", 
            CharSet::Testing => "0O|",
            CharSet::Light => {
                " .`'\",:;Il!i><~+_-?][}{1)(|\\/tfjrxnuvczXYUJCLQ0OZmwqpdbkhao*#MW&8%B@$"
            } // Sorted by density
            CharSet::Detailed => {
                "$@B%8&WM#*oahkbdpqwmZO0QLCJUYXzcvunxrjft/\\|()1{}[]?-_+~<>i!lI;:,\"^`'. "
            }
        }
    }
}
#[derive(Clone, ValueEnum)]
pub enum CharSet {
    Retro,
    Default,
    Light,
    Detailed,
    Testing
}

pub fn calculate_block_size(img_width: i32, width: i32) -> (u32, u32) {
    let block_w = img_width as f32 / width as f32;
    let block_h = block_w as f32 / 0.5;

    let block_w = block_w.max(1.0).round() as u32;
    let block_h = block_h.max(1.0).round() as u32;

    (block_w, block_h)
}

fn block_color(
    block_sample: &mut BlockSample,
    frame_data: &Mat,
    x0: u32,
    y0: u32,
    w: u32,
    h: u32,
) -> Result<()> {
    let img_w = frame_data.size()?.width;
    let img_h = frame_data.size()?.width;

    let mut r_sum = 0u32;
    let mut g_sum = 0u32;
    let mut b_sum = 0u32;
    let mut count = 0u32;

    let y_end = (y0 + h).min(img_h as u32);
    let x_end = (x0 + w).min(img_w as u32);

    for y in y0..y_end {
        for x in x0..x_end {
            let pixel = frame_data.at_2d::<Vec3b>(y as i32, x as i32)?;

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

    block_sample.lum = brightness;
    block_sample.r = r;
    block_sample.g = g;
    block_sample.b = r;

    Ok(())
}

fn get_blocks_data(
    frame_data: &Mat,
    blocks_data: &mut Vec<Vec<BlockSample>>,
    block_size: (u32, u32),
) -> Result<()> {
    let img_w = frame_data.size()?.width;
    let img_h = frame_data.size()?.height;

    let block_w = block_size.0;
    let block_h = block_size.1;

    for y0 in (0..img_h).step_by(block_h as usize) {
        let mut row: Vec<BlockSample> = Vec::new();
        for x0 in (0..img_w).step_by(block_w as usize) {
            let mut block_sample = BlockSample::default();
            block_color(
                &mut block_sample,
                frame_data,
                x0 as u32,
                y0 as u32,
                block_w as u32,
                block_h as u32,
            )?;

            row.push(block_sample);
        }
        blocks_data.push(row);
    }
    Ok(())
}

pub fn assign_chars(
    ascii_data: &mut Vec<Vec<(BlockSample, char)>>,
    mut blocks_data: Vec<Vec<BlockSample>>,
    char_set: &str,
    frame_data: &Mat,
    width: i32,
) -> Result<()> {
    let block_size = calculate_block_size(frame_data.size()?.width, width);

    get_blocks_data(&frame_data, &mut blocks_data, block_size)?;

    for rows in blocks_data {
        let mut row: Vec<(BlockSample, char)> = Vec::new();

        for element in rows {
            let lum = element.lum;
            let index = ((lum / 255.0) * ((char_set.len() - 1) as f32)) as usize;
            
            if let Some(ascii_char) = char_set.chars().nth(index) {
                row.push((element, ascii_char));
            }
        }

        ascii_data.push(row);
    }

    Ok(())
}
