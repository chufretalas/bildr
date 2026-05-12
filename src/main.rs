mod engine;
mod image;
mod kernel;

use std::path::PathBuf;

use clap::Parser;

use crate::engine::{Engine, Padding};

use crate::image::Image;
use crate::kernel::Kernel;

#[derive(Parser, Debug)]
struct Cli {
    /// Input file (ex: imagem.ppm)
    input: PathBuf,

    /// Output file (ex: resultado.png)
    output: PathBuf,

    /// REQUIRED: kernel file
    #[arg(short, long)]
    kernel: PathBuf,

    /// OPTIONAL: whether or not or not to normalize the kernel automatically
    #[arg(short, long)]
    normalize: bool,

    /// OPTIONAL: Which type of padding to use
    #[arg(short, long, value_enum, default_value_t=Padding::Zero)]
    padding: Padding,
}

fn main() {
    //TODO: Improve application layer errors
    let args = Cli::parse();

    let img = Image::from_file_path(&args.input).unwrap();

    let kernel = Kernel::from_file_path(&args.kernel, args.normalize).unwrap();

    let engine = Engine::new(args.padding);

    let out_img = engine.convolve(&img, &kernel);

    out_img.save_to_file(&args.output).unwrap();
}
