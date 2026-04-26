mod engine;
mod image;
mod kernel;

use crate::engine::{Engine, Padding};

use crate::image::Image;
use crate::kernel::Kernel;

fn main() {
    let blur_box_kernel = Kernel::new_normalized(3, 3, vec![1.0; 9]).unwrap();

    let sharpen_kernel = Kernel::new_normalized(
        3,
        3,
        vec![-1.0, -1.0, -1.0, -1.0, 9.0, -1.0, -1.0, -1.0, -1.0],
    )
    .unwrap();

    let edge_detection_kernel =
        Kernel::new_normalized(3, 3, vec![0.0, -1.0, 0.0, -1.0, 4.0, -1.0, 0.0, -1.0, 0.0])
            .unwrap();

    let img = Image::from_file_path("example_images/big.ppm".into()).unwrap();

    let engine = Engine::new(Padding::Zero);

    let out_img = engine.convolve(&img, &edge_detection_kernel);

    out_img.save_to_file("./test.ppm".into()).unwrap();
}
