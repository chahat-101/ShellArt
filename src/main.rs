use opencv::{Result, core, highgui, prelude::*, videoio};
use opencv::core::{Vec3b, Vec3d};

mod cli;

mod utils;
use utils::{
    get_frame_data
};


fn main() -> Result<()> {
    let mut cam = videoio::VideoCapture::new(0, videoio::CAP_ANY)?;

    

    if !cam.is_opened()? {
        panic!("unable to open camera");
    }


    highgui::named_window("Camera", highgui::WINDOW_AUTOSIZE);

    loop{

        let mut frame = Mat::default();

        get_frame_data(&mut cam,&mut frame ,true, true);

        if frame.size()?.width > 0{
            highgui::imshow("Camera", &frame)?;


        }



        let key = highgui::wait_key(1)?;
        if key == 'q' as i32{
            break;
        }

    }

    Ok(())
}
