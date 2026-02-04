use clap::{Parser, ValueEnum};
use crossterm::{
    cursor,
    event::{self, Event, KeyCode},
    style::{Color, Print, SetForegroundColor},
    terminal::{self},
    ExecutableCommand, QueueableCommand,
};
use opencv::imgproc;
use opencv::{core, highgui, prelude::*, videoio};
use opencv::core::{Size, Vec3b};
use std::io::{stdout, Write};
use rand::Rng;

#[derive(Clone, ValueEnum, Default, PartialEq)]
pub enum CharSet {
    Retro,
    #[default]
    Default,
    Light,
    Detailed,
    Testing,
    Testing2,
}

impl CharSet {
    pub fn get_chars(&self) -> &'static str {
        match self {
            CharSet::Retro => " ░▒▓█",
            CharSet::Default => "@%#*+=-:.",
            CharSet::Testing => "0O|",
            CharSet::Light => {
                r###" .`'",:;Il!i><~+_-?][}{1)(|\/tfjrxnuvczXYUJCLQ0OZmwqpdbkhao*#MW&8%B@$"###
            }
            CharSet::Testing2 => "01",
            CharSet::Detailed => "$@B%8&WM #*oahkbdpqwmZO0QLCJUYXzcvunxrjft/()1{}[]?-_+~<>i!lI;:,",
        }
    }
}

#[derive(Clone, ValueEnum, Default, PartialEq, Debug)]
pub enum ArtMode {
    #[default] Standard,   // Original Colors
    Grayscale,  // Black & White
    Matrix,     // Shades of Green
    Thermal,    // Heatmap (Blue -> Red)
    Amber,      // Retro Amber Monochrome
    Neon,       // Cyberpunk (Purple/Pink/Cyan)
    Rainbow,    // Animated Rainbow
    Cga,        // Cyan/Magenta/White/Black
    Glitch,     // Random artifacts
}

#[derive(Parser)]
pub struct Args {
    /// Flip the image horizontally
    #[arg(long, value_enum, default_value_t = CharSet::Default)]
    pub charset: CharSet,

    /// Rendering Mode
    #[arg(long, value_enum, default_value_t = ArtMode::Standard)]
    pub mode: ArtMode,

    #[arg(long, default_value_t = 300)]
    pub width: i32,

    /// Camera device index
    #[arg(long, default_value_t = 0)]
    pub device: i32,

    /// Render to terminal directly
    #[arg(long, default_value_t = false)]
    pub terminal: bool,
}

#[derive(Default, Clone, Copy)]
pub struct BlockSample {
    pub lum: f32,
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

pub const WEIGHTS: [f32; 3] = [0.299, 0.587, 0.114];

pub fn get_frame_data(
    cam: &mut videoio::VideoCapture,
    frame: &mut Mat,
    flipped: bool,
) -> opencv::Result<()> {
    cam.read(frame)?;

    if frame.empty() {
        return Ok(());
    }

    if flipped {
        let mut flipped_frame = Mat::default();
        core::flip(frame, &mut flipped_frame, 1)?;
        *frame = flipped_frame;
    }

    Ok(())
}

pub fn calculate_block_size(img_width: i32, width: i32) -> (u32, u32) {
    let block_w = (img_width as f32 / width.max(1) as f32).max(1.0);
    let block_h = (block_w / 0.5).max(1.0);

    (block_w.round() as u32, block_h.round() as u32)
}

pub fn assign_chars(
    ascii_data: &mut Vec<Vec<(BlockSample, char)>>,
    char_set: &str,
    frame_data: &Mat,
    width: i32,
) -> opencv::Result<()> {
    if frame_data.empty() {
        return Ok(());
    }

    let img_w = frame_data.size()?.width;
    let (block_w, block_h) = calculate_block_size(img_w, width);
    
    let target_width = (img_w as f32 / block_w as f32).floor() as i32;
    let target_height = (frame_data.size()?.height as f32 / block_h as f32).floor() as i32;

    if target_width <= 0 || target_height <= 0 {
        return Ok(());
    }

    let mut resized = Mat::default();
    imgproc::resize(
        frame_data,
        &mut resized,
        Size::new(target_width, target_height),
        0.0,
        0.0,
        imgproc::INTER_AREA,
    )?;

    let char_vec: Vec<char> = char_set.chars().collect();
    let char_count = char_vec.len();

    for y in 0..target_height {
        let mut row = Vec::with_capacity(target_width as usize);
        for x in 0..target_width {
            let pixel = resized.at_2d::<Vec3b>(y, x)?;
            let b = pixel[0];
            let g = pixel[1];
            let r = pixel[2];

            let lum = r as f32 * WEIGHTS[0] + g as f32 * WEIGHTS[1] + b as f32 * WEIGHTS[2];
            let index = ((lum / 256.0) * char_count as f32) as usize;
            let index = index.min(char_count - 1);

            let sample = BlockSample { lum, r, g, b };
            row.push((sample, char_vec[index]));
        }
        ascii_data.push(row);
    }

    Ok(())
}

// Helper to convert HSV to RGB (Hue 0-360, Sat 0-1, Val 0-1)
fn hsv_to_rgb(h: f32, s: f32, v: f32) -> (u8, u8, u8) {
    let c = v * s;
    let x = c * (1.0 - ((h / 60.0) % 2.0 - 1.0).abs());
    let m = v - c;

    let (r, g, b) = if h < 60.0 {
        (c, x, 0.0)
    } else if h < 120.0 {
        (x, c, 0.0)
    } else if h < 180.0 {
        (0.0, c, x)
    } else if h < 240.0 {
        (0.0, x, c)
    } else if h < 300.0 {
        (x, 0.0, c)
    } else {
        (c, 0.0, x)
    };

    (
        ((r + m) * 255.0) as u8,
        ((g + m) * 255.0) as u8,
        ((b + m) * 255.0) as u8,
    )
}

fn get_color(sample: &BlockSample, mode: &ArtMode, x: usize, y: usize, frame_count: usize) -> (u8, u8, u8) {
    match mode {
        ArtMode::Standard => (sample.r, sample.g, sample.b),
        ArtMode::Grayscale => {
            let l = sample.lum as u8;
            (l, l, l)
        },
        ArtMode::Matrix => (0, sample.lum as u8, 0),
        ArtMode::Amber => {
            let l = sample.lum / 255.0;
            ((l * 255.0) as u8, (l * 176.0) as u8, 0)
        },
        ArtMode::Thermal => {
            let l = sample.lum;
            if l < 64.0 {
                (0, (l * 4.0) as u8, 255) // Blue -> Cyan
            } else if l < 128.0 {
                (0, 255, (255.0 - (l - 64.0) * 4.0) as u8) // Cyan -> Green
            } else if l < 192.0 {
                (((l - 128.0) * 4.0) as u8, 255, 0) // Green -> Yellow
            } else {
                (255, (255.0 - (l - 192.0) * 4.0) as u8, 0) // Yellow -> Red
            }
        },
        ArtMode::Neon => {
            let l = sample.lum;
            if l < 85.0 {
                 // Deep Purple shadows
                 let factor = l / 85.0;
                 ((20.0 * factor) as u8, 0, (40.0 + 215.0 * factor) as u8)
            } else if l < 170.0 {
                // Hot Pink mid-tones
                let factor = (l - 85.0) / 85.0;
                (255, 0, (255.0 - 127.0 * factor) as u8)
            } else {
                // Cyan highlights
                let factor = (l - 170.0) / 85.0;
                ((255.0 * (1.0 - factor)) as u8, (255.0 * factor) as u8, 255)
            }
        },
        ArtMode::Rainbow => {
            let hue = (x as f32 * 2.0 + y as f32 * 4.0 + frame_count as f32 * 5.0) % 360.0;
            hsv_to_rgb(hue, 1.0, 1.0)
        },
        ArtMode::Cga => {
            let l = sample.lum;
            if l < 50.0 {
                (0, 0, 0) // Black
            } else if l < 120.0 {
                (255, 85, 255) // Magenta
            } else if l < 190.0 {
                (85, 255, 255) // Cyan
            } else {
                (255, 255, 255) // White
            }
        },
        ArtMode::Glitch => {
            let mut rng = rand::thread_rng();
            if rng.gen_bool(0.05) {
                // Random random color glitch
                (rng.r#gen(), rng.r#gen(), rng.r#gen())
            } else {
                 // Slight color shift
                 (sample.r.wrapping_add(10), sample.g, sample.b.wrapping_add(10))
            }
        }
    }
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let cam = videoio::VideoCapture::new(args.device, videoio::CAP_ANY)?;

    if !cam.is_opened().map_err(|e| anyhow::anyhow!(e))? {
        return Err(anyhow::anyhow!("Could not open video device {}", args.device));
    }

    if args.terminal {
        run_terminal_mode(cam, args)?;
    } else {
        run_gui_mode(cam, args)?;
    }

    Ok(())
}

fn run_terminal_mode(mut cam: videoio::VideoCapture, args: Args) -> anyhow::Result<()> {
    let mut stdout = stdout();
    terminal::enable_raw_mode()?;
    stdout.execute(terminal::EnterAlternateScreen)?;
    stdout.execute(cursor::Hide)?;

    let char_set_str = CharSet::get_chars(&args.charset);
    let char_vec: Vec<char> = char_set_str.chars().collect();
    let width = args.width;

    let mut frame = Mat::default();
    let mut frame_count = 0;
    let mut rng = rand::thread_rng();

    loop {
        if event::poll(std::time::Duration::from_millis(1))? {
            if let Event::Key(key) = event::read()? {
                if key.code == KeyCode::Char('q') || key.code == KeyCode::Esc {
                    break;
                }
            }
        }

        get_frame_data(&mut cam, &mut frame, true).map_err(|e| anyhow::anyhow!(e))?;

        if frame.empty() {
            continue;
        }

        let mut ascii_data: Vec<Vec<(BlockSample, char)>> = Vec::new();
        assign_chars(&mut ascii_data, char_set_str, &frame, width).map_err(|e| anyhow::anyhow!(e))?;

        // Reset cursor to top-left
        stdout.queue(cursor::MoveTo(0, 0))?;

        for (y, row) in ascii_data.iter().enumerate() {
            for (x, (sample, ch)) in row.iter().enumerate() {
                
                let (r, g, b) = get_color(sample, &args.mode, x, y, frame_count);
                
                let final_char = if args.mode == ArtMode::Glitch && rng.gen_bool(0.02) {
                    char_vec[rng.gen_range(0..char_vec.len())]
                } else {
                    *ch
                };

                stdout.queue(SetForegroundColor(Color::Rgb { r, g, b }))?;
                stdout.queue(Print(final_char))?;
            }
            stdout.queue(Print("\r\n"))?;
        }

        stdout.flush()?;
        frame_count = frame_count.wrapping_add(1);
    }

    // Cleanup
    stdout.execute(cursor::Show)?;
    stdout.execute(terminal::LeaveAlternateScreen)?;
    terminal::disable_raw_mode()?;
    Ok(())
}

fn run_gui_mode(mut cam: videoio::VideoCapture, args: Args) -> anyhow::Result<()> {
    let char_set_str = CharSet::get_chars(&args.charset);
    let char_vec: Vec<char> = char_set_str.chars().collect();
    let width = args.width;

    highgui::named_window("Camera", highgui::WINDOW_AUTOSIZE).map_err(|e| anyhow::anyhow!(e))?;
    highgui::named_window("Ascii-art ", highgui::WINDOW_AUTOSIZE).map_err(|e| anyhow::anyhow!(e))?;

    let mut frame = Mat::default();
    let mut ascii_frame = Mat::default();
    let mut frame_count = 0;
    let mut rng = rand::thread_rng();

    loop {
        get_frame_data(&mut cam, &mut frame, true).map_err(|e| anyhow::anyhow!(e))?;

        if frame.empty() {
            continue;
        }

        highgui::imshow("Camera", &frame).map_err(|e| anyhow::anyhow!(e))?;

        let mut ascii_data: Vec<Vec<(BlockSample, char)>> = Vec::new();
        assign_chars(&mut ascii_data, char_set_str, &frame, width).map_err(|e| anyhow::anyhow!(e))?;

        if ascii_frame.size().map_err(|e| anyhow::anyhow!(e))? != frame.size().map_err(|e| anyhow::anyhow!(e))?
            || ascii_frame.typ() != frame.typ()
        {
            ascii_frame = Mat::new_rows_cols_with_default(
                frame.rows(),
                frame.cols(),
                frame.typ(),
                core::Scalar::all(0.0),
            )
            .map_err(|e| anyhow::anyhow!(e))?;
        } else {
            ascii_frame
                .set_to(&core::Scalar::all(0.0), &core::no_array())
                .map_err(|e| anyhow::anyhow!(e))?;
        }

        let block_size = calculate_block_size(frame.size().map_err(|e| anyhow::anyhow!(e))?.width, width);

        for (y, row) in ascii_data.iter().enumerate() {
            for (x, (sample, ch)) in row.iter().enumerate() {
                
                let (r, g, b) = get_color(sample, &args.mode, x, y, frame_count);
                let color = opencv::core::Scalar::new(b as f64, g as f64, r as f64, 0.0);

                let final_char = if args.mode == ArtMode::Glitch && rng.gen_bool(0.02) {
                    char_vec[rng.gen_range(0..char_vec.len())].to_string()
                } else {
                    ch.to_string()
                };

                imgproc::put_text(
                    &mut ascii_frame,
                    &final_char,
                    opencv::core::Point::new(
                        (x as i32) * block_size.0 as i32,
                        (y as i32) * block_size.1 as i32 + block_size.1 as i32,
                    ),
                    imgproc::FONT_HERSHEY_PLAIN,
                    block_size.0 as f64 / 10.0,
                    color,
                    1,
                    imgproc::LINE_AA,
                    false,
                )
                .map_err(|e| anyhow::anyhow!(e))?;
            }
        }

        highgui::imshow("Ascii-art ", &ascii_frame).map_err(|e| anyhow::anyhow!(e))?;

        let key = highgui::wait_key(1).map_err(|e| anyhow::anyhow!(e))?;
        if key == 'q' as i32 {
            break;
        }
        frame_count = frame_count.wrapping_add(1);
    }
    Ok(())
}
