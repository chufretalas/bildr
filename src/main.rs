mod image;
mod kernel;

use image::Image;

use kernel::Kernel;

fn main() {
    let k = Kernel::new(0.1, 3, 3, vec![1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0]);
    dbg!("{}", &k);
    let img = Image::from_file_path("example_images/hk.ppm".into()).unwrap();
    // dbg!("{}", &img);
    img.save_to_file("./test.ppm".into()).unwrap();
}
