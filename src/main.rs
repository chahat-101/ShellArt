use clap::Parser;
use image::GenericImageView;
use image::imageops::FilterType;

const WEIGHTS: [f32; 3] = [0.299, 0.587, 0.114];

#[derive(Parser)]
struct Args {
    #[arg(long, alias = "p")]
    path: String,
    #[arg(long, default_value_t = 100)]
    width: u32,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let path = args.path;
    let target_width = args.width;

    let ascii_chars: Vec<char> = "Ña#W$9876543210?!abc;:+=-,._".chars().collect();

    let img = image::open(path)?;
    let (orig_height, orig_width) = img.dimensions();

    let aspect_ratio = (orig_height as f32) / (orig_width as f32);
    let target_height = (orig_height as f32 * aspect_ratio * 0.5) as u32;

    let resized_img = img.resize_exact(target_width, target_height, FilterType::Triangle);

    let rgba_8 = resized_img.to_rgba8();
    let width = resized_img.width();

    println!("{:?} , {:?}", img.dimensions(), resized_img.dimensions());

    for (x, _y, pixel) in rgba_8.enumerate_pixels() {
        let brightness = WEIGHTS[0] * pixel[0] as f32 + WEIGHTS[1] * pixel[1] as f32;

        let index = ((brightness / 255.0) * 27.0) as usize;

        let ascii_char = ascii_chars[index];
        print!("{}", ascii_char);

        if x == width - 1 {
            println!();
        }
    }

    Ok(())
}

fn gray_scale_converter(
    img_path: &str,
    save_path: &mut str,
) -> Result<(), Box<dyn std::error::Error>> {
    let img = image::open(img_path)?;

    let mut rgba8_img = img.to_rgba8();

    println!("{:?}", img.color());

    for (_x, _y, pixel) in rgba8_img.enumerate_pixels_mut() {
        let (r, g, b, _a) = (pixel[0], pixel[1], pixel[2], pixel[3]);

        let gray_scale_point =
            r as f32 * WEIGHTS[0] + g as f32 * WEIGHTS[1] + b as f32 * WEIGHTS[2];

        pixel[0] = gray_scale_point as u8;
        pixel[1] = gray_scale_point as u8;
        pixel[2] = gray_scale_point as u8;
    }

    let save_path = format!("{}.png", save_path);

    rgba8_img.save(save_path).expect("failed to save");

    Ok(())
}
