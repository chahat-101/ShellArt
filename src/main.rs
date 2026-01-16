use clap::{Parser, ValueEnum};
use colored::*;
use image::RgbImage;
#[derive(Parser)]
struct Args {
    #[arg(long, alias = "p")]
    path: String,

    #[arg(long, value_enum, default_value_t = CharSet::Default)]
    charset: CharSet,
    #[arg(long, default_value_t = 100)]
    width: u32,
    #[arg(long, short, default_value_t = false)]
    invert: bool,
}

impl CharSet {
    fn get_chars(&self) -> &'static str {
        match self {
            CharSet::Retro => " ░▒▓█",
            CharSet::Default => " .:-=+*#%@",
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
enum CharSet {
    Retro,
    Default,
    Light,
    Detailed,
}

const WEIGHTS: [f32; 3] = [0.299, 0.587, 0.114];
struct BlockSample {
    lum: f32,
    r: u8,
    b: u8,
    g: u8,
}

fn block_color(img: &RgbImage, x0: u32, y0: u32, w: u32, h: u32) -> BlockSample {
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

fn calculate_block_size(
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

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let path = args.path;
    let target_width = args.width;

    let chars_str = args.charset.get_chars();

    let ascii_chars: Vec<char> = if args.invert {
        chars_str.chars().rev().collect()
    } else {
        chars_str.chars().collect()
    };

    let img = image::open(path)?.to_rgb8();
    let (img_width, img_height) = img.dimensions();

    let aspect = 0.5;
    let (block_w, block_h) = calculate_block_size(img_width, img_height, target_width, aspect);

    let mut blocks_data: Vec<Vec<BlockSample>> = Vec::new();
    for y0 in (0..img_height).step_by(block_h as usize) {
        let mut row: Vec<BlockSample> = Vec::new();
        for x0 in (0..img_width).step_by(block_w as usize) {
            let block_sample = block_color(&img, x0, y0, block_w, block_h);

            row.push(block_sample);
        }
        blocks_data.push(row);
    }

    let mut ascii_art = String::new();
    for row in blocks_data {
        for block in row {
            let lum = block.lum;

            let index = ((lum / 255.0) * ((ascii_chars.len() - 1) as f32)).round() as usize;
            let character = ascii_chars[index].to_string();
            ascii_art.push_str(&character.truecolor(block.r, block.g, block.b).to_string());
        }
        ascii_art.push('\n');
    }

    println!("{}", ascii_art);
    Ok(())
}
