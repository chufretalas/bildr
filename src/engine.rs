use crate::{
    image::{Image, Pixel},
    kernel::Kernel,
};

pub enum Padding {
    Zero,
    // Clamp, // TODO
    // Reflect, // TODO
}

pub struct Engine {
    padding: Padding,
}

impl Engine {
    pub fn new(padding: Padding) -> Self {
        Engine { padding }
    }

    pub fn convolve(&self, input_img: &Image, kernel: &Kernel) -> Image {
        let w = input_img.width();
        let h = input_img.height();
        let mut res_img = Image::empty(w, h);

        let start_y = -(kernel.anchor_y() as i32);
        let end_y = kernel.height() as i32 - kernel.anchor_y() as i32;

        let start_x = -(kernel.anchor_x() as i32);
        let end_x = kernel.width() as i32 - kernel.anchor_x() as i32;

        // Looping through the image
        for y in 0..(h as i32) {
            for x in 0..(w as i32) {
                let mut new_r = 0.0;
                let mut new_g = 0.0;
                let mut new_b = 0.0;

                // Looping through the kernel for each pixel
                for dy in start_y..end_y {
                    for dx in start_x..end_x {
                        let pixel = match input_img.get_pixel(x + dx, y + dy) {
                            Some(p) => p,
                            None => match self.padding {
                                Padding::Zero => Pixel::black(),
                            },
                        };

                        let weight = kernel.get_weight(dx, dy).unwrap();
                        new_r += pixel.r() as f32 * weight;
                        new_g += pixel.g() as f32 * weight;
                        new_b += pixel.b() as f32 * weight;
                    }
                }

                new_r *= kernel.scaling_factor();
                new_g *= kernel.scaling_factor();
                new_b *= kernel.scaling_factor();

                let res_pixel = res_img.get_pixel_mut(x, y).unwrap();
                res_pixel.set_rgb(
                    new_r.clamp(0.0, 255.0) as u8,
                    new_g.clamp(0.0, 255.0) as u8,
                    new_b.clamp(0.0, 255.0) as u8,
                );
            }
        }

        res_img
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use std::fs;
    use std::path::PathBuf;

    fn get_test_image_path(filename: &str) -> String {
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push("test_data/imgs");
        path.push(filename);
        path.to_str()
            .expect("Path contains invalid unicode")
            .to_string()
    }

    #[test]
    fn test_convolution_identity_kernel() {
        let engine = Engine::new(Padding::Zero);

        let mut input_img = Image::empty(3, 3);
        for y in 0..3 {
            for x in 0..3 {
                input_img.get_pixel_mut(x, y).unwrap().set_rgb(100, 50, 25);
            }
        }

        let identity_weights = vec![0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0];
        let kernel = Kernel::new(1.0, 3, 3, identity_weights).unwrap();

        let output_img = engine.convolve(&input_img, &kernel);

        for y in 0..3 {
            for x in 0..3 {
                let original_pixel = input_img.get_pixel(x, y).unwrap();
                let convoluted_pixel = output_img.get_pixel(x, y).unwrap();
                assert_eq!(original_pixel, convoluted_pixel);
            }
        }
    }

    #[test]
    fn test_convolution_edge_detection_padding_zero() {
        let input_path = get_test_image_path("100x100_p6.ppm");

        let output_temp_path = env::temp_dir().join(format!("test_out_{}.ppm", std::process::id()));
        let output_temp_path_str = output_temp_path.to_str().unwrap().to_string();

        let reference_path = get_test_image_path("100x100_p6_edge_detection.ppm");

        let engine = Engine::new(Padding::Zero);

        let input_img = Image::from_file_path(input_path).unwrap();

        let edge_detection_kernel =
            Kernel::new_normalized(3, 3, vec![0.0, -1.0, 0.0, -1.0, 4.0, -1.0, 0.0, -1.0, 0.0])
                .unwrap();

        let output_img = engine.convolve(&input_img, &edge_detection_kernel);

        output_img
            .save_to_file(output_temp_path_str.clone())
            .expect("Failed to save temp file");

        let generated_bytes = fs::read(&output_temp_path).expect("Failed to read generated file");
        let reference_bytes = fs::read(&reference_path).expect("Failed to read reference file");

        assert_eq!(
            generated_bytes, reference_bytes,
            "The generated image bytes do not match the golden reference!"
        );

        let _ = fs::remove_file(output_temp_path);
    }
}
