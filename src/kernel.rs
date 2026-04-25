#[derive(Debug, Clone)]
pub struct Kernel {
    scaling_factor: f32,
    width: u32,
    height: u32,
    weights: Vec<f32>,

    // The kernel's center (which is offset one-down one-right if the kernel's dimensions are even)
    anchor_x: u32,
    anchor_y: u32,
}

impl Kernel {
    // TODO add a Kernel::from_file
    
    pub fn new(
        scaling_factor: f32,
        width: u32,
        height: u32,
        weights: Vec<f32>,
    ) -> Result<Self, &'static str> {
        if width * height != weights.len() as u32 {
            Err("The amount of weights does not match width and height")
        } else {
            Ok(Self {
                scaling_factor,
                width,
                height,
                weights,
                anchor_x: width / 2,
                anchor_y: height / 2,
            })
        }
    }

    pub fn new_normalized(width: u32, height: u32, weights: Vec<f32>) -> Result<Self, &'static str> {
        let scaling_factor: f32 = {
            let sum: f32 = weights.iter().sum();
            if sum == 0.0 {
                1.0
            } else {
                1.0 / sum
            }
        };
        Self::new(scaling_factor, width, height, weights)
    }

    /// Returns the weight at a relative position to the anchor point
    /// dx and dy are center relative positions, so dx=0 and dy=0 would be the kernel's center
    #[inline]
    pub fn get_weight(&self, dx: i32, dy: i32) -> Option<f32> {
        let absolute_x = self.anchor_x as i32 + dx;
        if absolute_x < 0 || absolute_x >= self.width as i32 {
            return None;
        }

        let absolute_y = self.anchor_y as i32 + dy;
        if absolute_y < 0 || absolute_y >= self.height as i32 {
            return None;
        }

        Some(self.weights[absolute_y as usize * (self.width as usize) + absolute_x as usize])
    }
}
