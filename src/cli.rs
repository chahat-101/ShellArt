use clap::{Parser,ValueEnum};

#[derive(Parser)]
struct Args {
    /// Flip the image horizontally
    #[arg(long)]
    charset: bool,

    /// Convert frame to grey scale
    #[arg(long)]
    grey_scale: bool,

    /// Camera device index
    #[arg(long, default_value_t = 0)]
    device: i32,
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