use clap::Parser;
use colored::*;
mod utils;
use utils::{BlockSample, block_color, calculate_block_size};

#[derive(Parser)]
struct Args {
    #[arg(long, alias = "p")]
    path: String,
    #[arg(long, default_value_t = 100)]
    width: u32,
}

enum CharSet {
    Retro,
    Default,
    Light,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let path = args.path;
    let target_width = args.width;

    //let ascii_chars: Vec<char> = "Ña#W$9876543210?!abc;:+=-,._".chars().collect();
    let ascii_chars: Vec<char> = " ░▒▓█".chars().collect();
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

            let index = ((lum / 255.0) * (4.0)).round() as usize;
            let character = ascii_chars[index].to_string();
            ascii_art.push_str(&character.truecolor(block.r, block.g, block.b).to_string());
        }
        ascii_art.push('\n');
    }

    println!("{}", ascii_art);

    Ok(())
}
