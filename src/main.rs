mod engine;
mod image;
mod kernel;

use std::path::PathBuf;
use std::sync::mpsc::channel;
use std::thread;

use clap::Parser;

use crate::engine::{Engine, Padding};

use crate::image::{Image, Pixel};
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

    /// OPTIONAL: Opens a windows to visualize the convolution process
    #[arg(short, long)]
    visualize: bool,
}

fn main() {
    //TODO: Improve application layer errors
    let args = Cli::parse();

    let img = Image::from_file_path(&args.input).unwrap();

    let kernel = Kernel::from_file_path(&args.kernel, args.normalize).unwrap();

    let engine = Engine::new(args.padding);

    let out_img = if args.visualize {
        let mut out_img = img.clone();

        let (tx, rx) = channel::<(usize, Vec<Pixel>)>();
        let compute_handle = thread::spawn(move || {
            engine.convolve_with_channel(&img, &kernel, tx);
        });

        for _ in 0..out_img.height() {
            let (y, line_data) = rx.recv().unwrap();

            // TODO: write result to the frame buffer

            // Build the final img
            for x in 0..out_img.width() as i32 {
                let _ = out_img.set_pixel(x, y as i32, line_data[x as usize]);
            }
        }

        compute_handle.join().expect("Compute thread panicked!");

        out_img
    } else {
        engine.convolve(&img, &kernel)
    };

    out_img.save_to_file(&args.output).unwrap();
}
