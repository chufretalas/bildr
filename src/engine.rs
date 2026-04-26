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
