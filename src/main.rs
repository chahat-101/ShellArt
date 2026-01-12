use clap::Parser;
use image::GenericImageView;
use std::collections::HashMap;

const WEIGHTS: [f32; 3] = [0.299, 0.587, 0.114];

#[derive(Parser)]
struct Args {
    #[arg(long, alias = "p")]
    path: String,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let path = args.path;

    let ascii_chars: Vec<char> = "Ña#W$9876543210?!abc;:+=-,._".chars().collect();

    let brightness_point = 10.0;

    let img = image::open(path)?;

    let rgba_8 = img.to_rgba8();

    let width = img.width();

    for (x, _y, pixel) in rgba_8.enumerate_pixels() {
        let brightness = WEIGHTS[0] * pixel[0] as f32
            + WEIGHTS[1] * pixel[1] as f32
            + WEIGHTS[2] * pixel[2] as f32;
        let index = ((brightness / 255.0) * 27.0) as usize;

        let ascii_char = ascii_chars[index] as char;

        print!("{}", ascii_char);

        if x == width - 1 {
            println!();
        }
    }

    Ok(())
}

fn gray_scale_converter(path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let img = image::open(path)?;

    let mut rgba8_img = img.to_rgba8();

    println!("{:?}", img.color());

    for (x, y, pixel) in rgba8_img.enumerate_pixels_mut() {
        let (r, g, b, a) = (pixel[0], pixel[1], pixel[2], pixel[3]);

        let gray_scale_point =
            r as f32 * WEIGHTS[0] + g as f32 * WEIGHTS[1] + b as f32 * WEIGHTS[2];

        pixel[0] = gray_scale_point as u8;
        pixel[1] = gray_scale_point as u8;
        pixel[2] = gray_scale_point as u8;
    }

    rgba8_img.save("output2.png").expect("failed to save");

    Ok(())
}
