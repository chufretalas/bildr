use crate::{
    image::{Image, Pixel},
    kernel::Kernel,
};

pub enum Padding {
    Zero,
    Clamp,
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
        let mut res_img = Image::black(w, h);

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
                                Padding::Clamp => {
                                    let target_x = x + dx;
                                    let target_y = y + dy;

                                    let clamped_x = if target_x < 0 {
                                        0
                                    } else if target_x >= input_img.width() as i32 {
                                        (input_img.width() - 1) as i32
                                    } else {
                                        target_x
                                    };

                                    let clamped_y = if target_y < 0 {
                                        0
                                    } else if target_y >= input_img.height() as i32 {
                                        (input_img.height() - 1) as i32
                                    } else {
                                        target_y
                                    };

                                    input_img.get_pixel(clamped_x, clamped_y).unwrap()
                                }
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

        let mut input_img = Image::black(3, 3);
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
                assert_eq!(
                    original_pixel, convoluted_pixel,
                    "Mismatch at x: {}, y: {}",
                    x, y
                );
            }
        }
    }

    #[test]
    fn test_convolution_padding_clamp_left_border() {
        let engine = Engine::new(Padding::Clamp);

        // 2x2 image
        //  red  | black
        // green | black
        let mut input_img = Image::black(2, 2);
        input_img.get_pixel_mut(0, 0).unwrap().set_rgb(255, 0, 0);
        input_img.get_pixel_mut(0, 1).unwrap().set_rgb(0, 255, 0);

        let kernel = Kernel::new(1.0, 5, 1, vec![1.0, 0.0, 0.0, 0.0, 0.0]).unwrap();

        let output_img = engine.convolve(&input_img, &kernel);

        // Expect
        //  red  |  red
        // green | green
        let mut ref_img = Image::black(3, 3);
        ref_img.get_pixel_mut(0, 0).unwrap().set_rgb(255, 0, 0);
        ref_img.get_pixel_mut(1, 0).unwrap().set_rgb(255, 0, 0);
        ref_img.get_pixel_mut(0, 1).unwrap().set_rgb(0, 255, 0);
        ref_img.get_pixel_mut(1, 1).unwrap().set_rgb(0, 255, 0);

        for y in 0..2 {
            for x in 0..2 {
                let reference_pixel = ref_img.get_pixel(x, y).unwrap();
                let convoluted_pixel = output_img.get_pixel(x, y).unwrap();
                assert_eq!(
                    reference_pixel, convoluted_pixel,
                    "Mismatch at x: {}, y: {}",
                    x, y
                );
            }
        }
    }

    #[test]
    fn test_convolution_box_blur_clamp() {
        let engine = Engine::new(Padding::Clamp);

        // 5x1 image: A steep gradient on the left edge
        // Red values: [100, 50, 10, 0, 0]
        let mut input_img = Image::black(5, 1);
        input_img.get_pixel_mut(0, 0).unwrap().set_rgb(100, 0, 0);
        input_img.get_pixel_mut(1, 0).unwrap().set_rgb(50, 0, 0);
        input_img.get_pixel_mut(2, 0).unwrap().set_rgb(10, 0, 0);

        // 5x1 Box Blur
        let kernel = Kernel::new(0.2, 5, 1, vec![1.0, 1.0, 1.0, 1.0, 1.0]).unwrap();

        let output_img = engine.convolve(&input_img, &kernel);

        let target_pixel = output_img.get_pixel(0, 0).unwrap();

        assert_eq!(target_pixel.r(), 72);
    }

    #[test]
    fn test_convolution_padding_clamp_all_borders_deep() {
        let engine = Engine::new(Padding::Clamp);

        // 3x3 Image. We will paint the middle pixel of each edge a unique color.
        // Center and corners remain black.
        let mut input_img = Image::black(3, 3);
        input_img.get_pixel_mut(1, 0).unwrap().set_rgb(255, 0, 0); // Top edge: Red
        input_img.get_pixel_mut(1, 2).unwrap().set_rgb(0, 255, 0); // Bottom edge: Green
        input_img.get_pixel_mut(0, 1).unwrap().set_rgb(0, 0, 255); // Left edge: Blue
        input_img.get_pixel_mut(2, 1).unwrap().set_rgb(255, 255, 0); // Right edge: Yellow

        // --- 1. Test TOP Border ---
        #[rustfmt::skip]
        let look_up = Kernel::new(
            1.0,
            5,
            5,
            vec![
                0.0, 0.0, 1.0, 0.0, 0.0,
                0.0, 0.0, 0.0, 0.0, 0.0,
                0.0, 0.0, 0.0, 0.0, 0.0,
                0.0, 0.0, 0.0, 0.0, 0.0, 
                0.0, 0.0, 0.0, 0.0, 0.0,
            ],
        )
        .unwrap();
        let out_up = engine.convolve(&input_img, &look_up);

        let top_pixel = out_up.get_pixel(1, 0).unwrap();
        assert_eq!(
            top_pixel.r(),
            255,
            "Top border clamp failed! Looked 2px out of bounds and didn't get Red."
        );

        // --- 2. Test BOTTOM Border ---
        #[rustfmt::skip]
        let look_down = Kernel::new(
            1.0,
            5,
            5,
            vec![
                0.0, 0.0, 0.0, 0.0, 0.0,
                0.0, 0.0, 0.0, 0.0, 0.0,
                0.0, 0.0, 0.0, 0.0, 0.0,
                0.0, 0.0, 0.0, 0.0, 0.0, 
                0.0, 0.0, 1.0, 0.0, 0.0,
            ],
        )
        .unwrap();
        let out_down = engine.convolve(&input_img, &look_down);

        let bottom_pixel = out_down.get_pixel(1, 2).unwrap();
        assert_eq!(
            bottom_pixel.g(),
            255,
            "Bottom border clamp failed! Looked 2px out of bounds and didn't get Green."
        );

        // --- 3. Test LEFT Border ---
        #[rustfmt::skip]
        let look_left = Kernel::new(
            1.0,
            5,
            5,
            vec![
                0.0, 0.0, 0.0, 0.0, 0.0,
                0.0, 0.0, 0.0, 0.0, 0.0,
                1.0, 0.0, 0.0, 0.0, 0.0,
                0.0, 0.0, 0.0, 0.0, 0.0, 
                0.0, 0.0, 0.0, 0.0, 0.0,
            ],
        )
        .unwrap();
        let out_left = engine.convolve(&input_img, &look_left);

        let left_pixel = out_left.get_pixel(0, 1).unwrap();
        assert_eq!(
            left_pixel.b(),
            255,
            "Left border clamp failed! Looked 2px out of bounds and didn't get Blue."
        );

        // --- 4. Test RIGHT Border ---
        #[rustfmt::skip]
        let look_right = Kernel::new(
            1.0,
            5,
            5,
            vec![
                0.0, 0.0, 0.0, 0.0, 0.0, 
                0.0, 0.0, 0.0, 0.0, 0.0, 
                0.0, 0.0, 0.0, 0.0, 1.0, 
                0.0, 0.0, 0.0, 0.0, 0.0, 
                0.0, 0.0, 0.0, 0.0, 0.0,
            ],
        )
        .unwrap();
        let out_right = engine.convolve(&input_img, &look_right);

        let right_pixel = out_right.get_pixel(2, 1).unwrap();
        assert_eq!(
            right_pixel.r(),
            255,
            "Right border clamp failed! (Red component missing)"
        );
        assert_eq!(
            right_pixel.g(),
            255,
            "Right border clamp failed! (Green component missing)"
        );
    }

    #[test]
    fn test_convolution_padding_clamp_all_corners_deep() {
        let engine = Engine::new(Padding::Clamp);

        // 3x3 Image. We will paint the four corners unique colors.
        let mut input_img = Image::black(3, 3);
        input_img.get_pixel_mut(0, 0).unwrap().set_rgb(255, 0, 0); // Top-Left: Red
        input_img.get_pixel_mut(2, 0).unwrap().set_rgb(0, 255, 0); // Top-Right: Green
        input_img.get_pixel_mut(0, 2).unwrap().set_rgb(0, 0, 255); // Bottom-Left: Blue
        input_img.get_pixel_mut(2, 2).unwrap().set_rgb(255, 255, 0); // Bottom-Right: Yellow

        // --- 1. Test TOP-LEFT Corner ---
        #[rustfmt::skip]
        let look_up_left = Kernel::new(
            1.0,
            5,
            5,
            vec![
                1.0, 0.0, 0.0, 0.0, 0.0,
                0.0, 0.0, 0.0, 0.0, 0.0,
                0.0, 0.0, 0.0, 0.0, 0.0,
                0.0, 0.0, 0.0, 0.0, 0.0,
                0.0, 0.0, 0.0, 0.0, 0.0,
            ],
        )
        .unwrap();
        let out_ul = engine.convolve(&input_img, &look_up_left);

        let top_left_pixel = out_ul.get_pixel(0, 0).unwrap();
        assert_eq!(
            top_left_pixel.r(),
            255,
            "Top-Left corner clamp failed! Looked out of bounds diagonally and didn't get Red."
        );

        // --- 2. Test TOP-RIGHT Corner ---
        #[rustfmt::skip]
        let look_up_right = Kernel::new(
            1.0,
            5,
            5,
            vec![
                0.0, 0.0, 0.0, 0.0, 1.0, 
                0.0, 0.0, 0.0, 0.0, 0.0, 
                0.0, 0.0, 0.0, 0.0, 0.0, 
                0.0, 0.0, 0.0, 0.0, 0.0, 
                0.0, 0.0, 0.0, 0.0, 0.0,
            ],
        )
        .unwrap();
        let out_ur = engine.convolve(&input_img, &look_up_right);

        let top_right_pixel = out_ur.get_pixel(2, 0).unwrap();
        assert_eq!(
            top_right_pixel.g(),
            255,
            "Top-Right corner clamp failed! Looked out of bounds diagonally and didn't get Green."
        );

        // --- 3. Test BOTTOM-LEFT Corner ---
        #[rustfmt::skip]
        let look_down_left = Kernel::new(
            1.0,
            5,
            5,
            vec![
                0.0, 0.0, 0.0, 0.0, 0.0,
                0.0, 0.0, 0.0, 0.0, 0.0,
                0.0, 0.0, 0.0, 0.0, 0.0,
                0.0, 0.0, 0.0, 0.0, 0.0,
                1.0, 0.0, 0.0, 0.0, 0.0,
            ],
        )
        .unwrap();
        let out_dl = engine.convolve(&input_img, &look_down_left);

        let bottom_left_pixel = out_dl.get_pixel(0, 2).unwrap();
        assert_eq!(
            bottom_left_pixel.b(),
            255,
            "Bottom-Left corner clamp failed! Looked out of bounds diagonally and didn't get Blue."
        );

        // --- 4. Test BOTTOM-RIGHT Corner ---
        #[rustfmt::skip]
        let look_down_right = Kernel::new(
            1.0,
            5,
            5,
            vec![
                0.0, 0.0, 0.0, 0.0, 0.0,
                0.0, 0.0, 0.0, 0.0, 0.0,
                0.0, 0.0, 0.0, 0.0, 0.0,
                0.0, 0.0, 0.0, 0.0, 0.0,
                0.0, 0.0, 0.0, 0.0, 1.0,
            ],
        )
        .unwrap();
        let out_dr = engine.convolve(&input_img, &look_down_right);

        let bottom_right_pixel = out_dr.get_pixel(2, 2).unwrap();
        assert_eq!(
            bottom_right_pixel.r(),
            255,
            "Bottom-Right corner clamp failed! (Red missing)"
        );
        assert_eq!(
            bottom_right_pixel.g(),
            255,
            "Bottom-Right corner clamp failed! (Green missing)"
        );
    }

    #[test]
    fn test_convolution_padding_clamp_asymmetrical_out_of_bounds() {
        let engine = Engine::new(Padding::Clamp);

        // 3x3 Image with a Red Top-Left corner
        let mut input_img = Image::black(3, 3);
        input_img.get_pixel_mut(0, 0).unwrap().set_rgb(255, 0, 0);

        // 5x5 Kernel. Anchored at center (dx=0, dy=0).
        // We place the 1.0 at dx = -1, dy = -2.
        #[rustfmt::skip]
        let look_knight_move = Kernel::new(1.0, 5, 5, vec![
            0.0, 1.0, 0.0, 0.0, 0.0,
            0.0, 0.0, 0.0, 0.0, 0.0,
            0.0, 0.0, 0.0, 0.0, 0.0,
            0.0, 0.0, 0.0, 0.0, 0.0,
            0.0, 0.0, 0.0, 0.0, 0.0,
        ]).unwrap();

        let output_img = engine.convolve(&input_img, &look_knight_move);

        // Evaluate at x=0, y=0.
        // The kernel will ask for x = 0 + (-1) = -1
        // The kernel will ask for y = 0 + (-2) = -2
        // It should clamp to (0, 0), which is Red.
        let top_left_pixel = output_img.get_pixel(0, 0).unwrap();
        assert_eq!(
            top_left_pixel.r(),
            255,
            "Asymmetrical clamp failed! (-1, -2) did not clamp back to (0, 0)."
        );
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
