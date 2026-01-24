use opencv::{Result, core, highgui, prelude::*, videoio};
use opencv::imgproc;
use opencv::core::{Vec3b, Vec3d};
use clap::Parser;
mod cli;
use cli::{Args};

mod utils;
use utils::{
    get_frame_data,
    CharSet
};

use crate::utils::{BlockSample, assign_chars,calculate_block_size};

fn main() -> Result<()> {
    let mut cam = videoio::VideoCapture::new(0, videoio::CAP_ANY)?;

    let args = Args::parse();
    let char_set = CharSet::get_chars(&args.charset);
    let width = args.width;

    if !cam.is_opened()? {
        panic!("unable to open camera");
    }

    highgui::named_window("Camera", highgui::WINDOW_NORMAL)?;

    loop{

        let mut frame = Mat::default();
        get_frame_data(&mut cam,&mut frame ,true, true)?;

        let mut ascii_data:Vec<Vec<(BlockSample,char)>>= Vec::new();
        let mut blocks_data : Vec<Vec<BlockSample>> = Vec::new(); 

       assign_chars(&mut ascii_data, blocks_data, char_set, &frame, width)?;

        if frame.size()?.width > 0{
            highgui::imshow("Camera", &frame)?;
        }

        let block_size = calculate_block_size(frame.size()?.width, width);

        for (y, row) in ascii_data.iter().enumerate() {
            for (x, (_, ch)) in row.iter().enumerate() {
                let text = ch.to_string();

                imgproc::put_text(
                    &mut frame,
                    &text,
                    opencv::core::Point::new(
                        (x as i32) * block_size.0 as i32,
                        (y as i32) * block_size.1 as i32,
                    ),
                    imgproc::FONT_HERSHEY_PLAIN,
                    1.0,
                    opencv::core::Scalar::new(255.0, 255.0, 255.0, 0.0),
                    1,
                    imgproc::LINE_8,
                    false,
                )?;
            }
        }
        highgui::named_window("Ascii-art", highgui::WINDOW_NORMAL)?;
        highgui::imshow("Ascii-art", &frame)?;


        let key = highgui::wait_key(1)?;
        if key == 'q' as i32{
            break;
        }
    }

    Ok(())
}
