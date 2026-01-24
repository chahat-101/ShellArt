use clap::{Parser,ValueEnum};
use crate::utils;
use utils::CharSet;

#[derive(Parser)]
pub struct Args {
    /// Flip the image horizontally
    #[arg(long,value_enum,default_value_t = CharSet::Default)]
    pub charset: CharSet,

    /// Convert frame to grey scale
    #[arg(long)]
    pub grey_scale: bool,

    #[arg(long,default_value_t = 300)]
    pub width: i32,

    /// Camera device index
    #[arg(long, default_value_t = 0)]
    pub device: i32,
}

