use opencv::{Result, core, highgui, prelude::*, videoio};
use opencv::videoio::VideoCapture;
use opencv::core::{Vec3b};






pub const WEIGHTS:[f32;3] = [0.299,0.587,0.114];


pub fn get_frame_data(cam: &mut VideoCapture,frame:&mut Mat,flipped:bool,grey_scale:bool) -> Result<()>{

    
    cam.read(frame);

    if grey_scale{
        for row in 0..frame.rows(){
            for col in 0..frame.cols(){

                let mut pixel= frame.at_2d_mut::<Vec3b>(row, col)?;

                
                let grey_value = (
                    pixel[0] as f32 * WEIGHTS[2] +
                    pixel[1] as f32 * WEIGHTS[1] +
                    pixel[2] as f32 * WEIGHTS[0]
                ) as u8;

                pixel[0] = grey_value;
                pixel[1] = grey_value;
                pixel[2] = grey_value;
            
            }
        }
    }    

    if flipped{
        let mut flipped_frame = Mat::default();
        core::flip(frame,&mut flipped_frame, 1);
        *frame = flipped_frame;
        return Ok(());
    } 


    Ok(())

}


pub fn assign_chars() -> Result<()>{
    

    
    
    Ok(())
}