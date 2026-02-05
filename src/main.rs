use clap::{Parser, ValueEnum};
use crossterm::{
    cursor,
    event::{self, Event, KeyCode},
    style::{Color, Print, SetBackgroundColor, SetForegroundColor},
    terminal::{self},
    ExecutableCommand, QueueableCommand,
};
use opencv::imgproc;
use opencv::{core, prelude::*, videoio};
use opencv::core::{Size, Vec3b};
use std::io::{stdout, Write};
use rand::Rng;
use eframe::egui;

#[derive(Clone, ValueEnum, Default, PartialEq, Copy, Debug)]
pub enum CharSet {
    Retro,
    #[default]
    Default,
    Light,
    Detailed,
    Blocks,
    Binary,
    Minimalist,
    Modern,
    Slashed,
    Testing,
    Testing2,
}

impl CharSet {
    pub fn next(&self) -> Self {
        match self {
            CharSet::Retro => CharSet::Default,
            CharSet::Default => CharSet::Light,
            CharSet::Light => CharSet::Detailed,
            CharSet::Detailed => CharSet::Blocks,
            CharSet::Blocks => CharSet::Binary,
            CharSet::Binary => CharSet::Minimalist,
            CharSet::Minimalist => CharSet::Modern,
            CharSet::Modern => CharSet::Slashed,
            CharSet::Slashed => CharSet::Testing,
            CharSet::Testing => CharSet::Testing2,
            CharSet::Testing2 => CharSet::Retro,
        }
    }

    pub fn get_chars(&self) -> &'static str {
        match self {
            CharSet::Retro => " ░▒▓█",
            CharSet::Default => "@%#*+=-:.",
            CharSet::Testing => "0O|",
            CharSet::Light => {
                r###" .`'",:;Il!i><~+_-?][}{1)(|\/tfjrxnuvczXYUJCLQ0OZmwqpdbkhao*#MW&8%B@$"###
            }
            CharSet::Blocks => " ▏▎▍▌▋▊▉█",
            CharSet::Binary => "01",
            CharSet::Minimalist => " .·+",
            CharSet::Modern => " .:-=+*#%@",
            CharSet::Slashed => " /\\|",
            CharSet::Testing2 => "01",
            CharSet::Detailed => "$@B%8&WM #*oahkbdpqwmZO0QLCJUYXzcvunxrjft/()1{}[]?-_+~<>i!lI;:,",
        }
    }
}

#[derive(Clone, ValueEnum, Default, PartialEq, Debug, Copy)]
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

impl ArtMode {
    pub fn next(&self) -> Self {
        match self {
            ArtMode::Standard => ArtMode::Grayscale,
            ArtMode::Grayscale => ArtMode::Matrix,
            ArtMode::Matrix => ArtMode::Thermal,
            ArtMode::Thermal => ArtMode::Amber,
            ArtMode::Amber => ArtMode::Neon,
            ArtMode::Neon => ArtMode::Rainbow,
            ArtMode::Rainbow => ArtMode::Cga,
            ArtMode::Cga => ArtMode::Glitch,
            ArtMode::Glitch => ArtMode::Standard,
        }
    }
}

#[derive(Parser, Clone)]
pub struct Args {
    /// Charset to use
    #[arg(long, value_enum, default_value_t = CharSet::Default)]
    pub charset: CharSet,

    /// Rendering Mode
    #[arg(long, value_enum, default_value_t = ArtMode::Standard)]
    pub mode: ArtMode,

    /// Width of the output
    #[arg(long, default_value_t = 150)]
    pub width: i32,

    /// Camera device index
    #[arg(long, default_value_t = 0)]
    pub device: i32,

    /// Path to image or video file
    #[arg(short, long)]
    pub input: Option<String>,

    /// Flip the image horizontally
    #[arg(long, default_value_t = false)]
    pub flip: bool,

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
    let mut temp_frame = Mat::default();
    cam.read(&mut temp_frame)?;

    if temp_frame.empty() {
        // Try to loop if it's a file
        let _ = cam.set(videoio::CAP_PROP_POS_FRAMES, 0.0);
        cam.read(&mut temp_frame)?;
        if temp_frame.empty() {
            return Ok(());
        }
    }

    if flipped {
        core::flip(&temp_frame, frame, 1)?;
    } else {
        *frame = temp_frame;
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
    
    let cam = if let Some(input_path) = &args.input {
        videoio::VideoCapture::from_file(input_path, videoio::CAP_ANY)?
    } else {
        videoio::VideoCapture::new(args.device, videoio::CAP_ANY)?
    };

    if !cam.is_opened().map_err(|e| anyhow::anyhow!(e))? {
        return Err(anyhow::anyhow!("Could not open input: {:?}", args.input.as_deref().unwrap_or("camera")));
    }

    if args.terminal {
        run_terminal_mode(cam, args)?;
    } else {
        run_gui_mode(cam, args)?;
    }

    Ok(())
}

fn run_terminal_mode(mut cam: videoio::VideoCapture, mut args: Args) -> anyhow::Result<()> {
    let mut stdout = stdout();
    terminal::enable_raw_mode()?;
    stdout.execute(terminal::EnterAlternateScreen)?;
    stdout.execute(cursor::Hide)?;

    let mut width = args.width;
    let mut show_ui = true;
    let mut frame = Mat::default();
    let mut frame_count = 0;
    let mut rng = rand::thread_rng();

    loop {
        if event::poll(std::time::Duration::from_millis(1))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => break,
                    KeyCode::Char('m') => args.mode = args.mode.next(),
                    KeyCode::Char('c') => args.charset = args.charset.next(),
                    KeyCode::Char('+') | KeyCode::Char('=') => width = (width + 2).min(500),
                    KeyCode::Char('-') | KeyCode::Char('_') => width = (width - 2).max(10),
                    KeyCode::Char('h') => show_ui = !show_ui,
                    _ => {}
                }
            }
        }

        get_frame_data(&mut cam, &mut frame, args.flip).map_err(|e| anyhow::anyhow!(e))?;

        if frame.empty() {
            continue;
        }

        let char_set_str = args.charset.get_chars();
        let char_vec: Vec<char> = char_set_str.chars().collect();
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

        if show_ui {
            let (term_cols, term_rows) = terminal::size()?;
            
            // Top Help Overlay
            stdout.queue(cursor::MoveTo(0, 0))?;
            stdout.queue(SetForegroundColor(Color::White))?;
            stdout.queue(SetBackgroundColor(Color::Black))?;
            stdout.queue(Print(" [m]ode | [c]harset | [+/-] width | [h]ide UI | [q]uit "))?;
            
            // Bottom Status Bar
            if term_rows > 0 {
                stdout.queue(cursor::MoveTo(0, term_rows - 1))?;
                let status_text = format!(" MODE: {:?} | CHARSET: {:?} | WIDTH: {} ", args.mode, args.charset, width);
                let padding_len = (term_cols as usize).saturating_sub(status_text.len());
                let padding = " ".repeat(padding_len);
                
                stdout.queue(SetForegroundColor(Color::Black))?;
                stdout.queue(SetBackgroundColor(Color::White))?;
                stdout.queue(Print(format!("{}{}", status_text, padding)))?;
                stdout.queue(SetBackgroundColor(Color::Reset))?;
            }
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

struct ShellArtApp {
    cam: videoio::VideoCapture,
    mode: ArtMode,
    charset: CharSet,
    width: i32,
    flipped: bool,
    font_size: f32,
    frame_count: usize,
}

impl ShellArtApp {
    fn new(cam: videoio::VideoCapture, args: Args) -> anyhow::Result<Self> {
        Ok(Self {
            cam,
            mode: args.mode,
            charset: args.charset,
            width: args.width,
            flipped: args.flip,
            font_size: 8.0,
            frame_count: 0,
        })
    }
}

impl eframe::App for ShellArtApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::SidePanel::left("controls").show(ctx, |ui| {
            ui.heading("Controls");
            
            ui.add(egui::Slider::new(&mut self.width, 10..=400).text("Width"));
            ui.add(egui::Slider::new(&mut self.font_size, 2.0..=20.0).text("Font Size"));
            ui.checkbox(&mut self.flipped, "Flip Horizontal");
            
            ui.separator();
            ui.label("Mode:");
            ui.radio_value(&mut self.mode, ArtMode::Standard, "Standard");
            ui.radio_value(&mut self.mode, ArtMode::Grayscale, "Grayscale");
            ui.radio_value(&mut self.mode, ArtMode::Matrix, "Matrix");
            ui.radio_value(&mut self.mode, ArtMode::Thermal, "Thermal");
            ui.radio_value(&mut self.mode, ArtMode::Amber, "Amber");
            ui.radio_value(&mut self.mode, ArtMode::Neon, "Neon");
            ui.radio_value(&mut self.mode, ArtMode::Rainbow, "Rainbow");
            ui.radio_value(&mut self.mode, ArtMode::Cga, "CGA");
            ui.radio_value(&mut self.mode, ArtMode::Glitch, "Glitch");

            ui.separator();
            ui.label("Charset:");
            ui.radio_value(&mut self.charset, CharSet::Default, "Default");
            ui.radio_value(&mut self.charset, CharSet::Retro, "Retro");
            ui.radio_value(&mut self.charset, CharSet::Blocks, "Blocks");
            ui.radio_value(&mut self.charset, CharSet::Modern, "Modern");
            ui.radio_value(&mut self.charset, CharSet::Binary, "Binary");
            ui.radio_value(&mut self.charset, CharSet::Minimalist, "Minimalist");
            ui.radio_value(&mut self.charset, CharSet::Slashed, "Slashed");
            ui.radio_value(&mut self.charset, CharSet::Light, "Light");
            ui.radio_value(&mut self.charset, CharSet::Detailed, "Detailed");
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            let mut frame = Mat::default();
            if let Ok(_) = get_frame_data(&mut self.cam, &mut frame, self.flipped) {
                if !frame.empty() {
                    let mut ascii_data: Vec<Vec<(BlockSample, char)>> = Vec::new();
                    let char_set_str = self.charset.get_chars();
                    let char_vec: Vec<char> = char_set_str.chars().collect();
                    if let Ok(_) = assign_chars(&mut ascii_data, char_set_str, &frame, self.width) {
                        
                        let mut job = egui::text::LayoutJob::default();
                        let mut rng = rand::thread_rng();
                        
                        for (y, row) in ascii_data.iter().enumerate() {
                            for (x, (sample, ch)) in row.iter().enumerate() {
                                let (r, g, b) = get_color(sample, &self.mode, x, y, self.frame_count);
                                
                                let final_char = if self.mode == ArtMode::Glitch && rng.gen_bool(0.02) {
                                    char_vec[rng.gen_range(0..char_vec.len())]
                                } else {
                                    *ch
                                };

                                job.append(&final_char.to_string(), 0.0, egui::TextFormat {
                                    font_id: egui::FontId::monospace(self.font_size),
                                    color: egui::Color32::from_rgb(r, g, b),
                                    ..Default::default()
                                });
                            }
                            job.append("\n", 0.0, egui::TextFormat::default());
                        }

                        egui::ScrollArea::both().show(ui, |ui| {
                            ui.label(job);
                        });
                    }
                }
            }
            self.frame_count = self.frame_count.wrapping_add(1);
            ctx.request_repaint();
        });
    }
}

fn run_gui_mode(cam: videoio::VideoCapture, args: Args) -> anyhow::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([1280.0, 720.0]),
        ..Default::default()
    };
    
    eframe::run_native(
        "ShellArt GUI",
        options,
        Box::new(|_cc| {
            let app = ShellArtApp::new(cam, args).expect("Failed to initialize app");
            Box::new(app)
        }),
    ).map_err(|e| anyhow::anyhow!("eframe error: {}", e))
}
