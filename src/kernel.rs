use thiserror::Error;

#[derive(Debug, Error)]
pub enum KernelError {
    #[error("Dimension mismatch: expected {expected} weights (width * height), but got {actual}")]
    DimensionMismatch { expected: usize, actual: usize },

    #[error("IO Error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Failed to parse kernel scaling factor: {0}")]
    ParseScalingFactor(#[source] std::num::ParseFloatError),

    #[error("Failed to parse kernel weight value: {0}")]
    ParseWeight(#[source] std::num::ParseFloatError),
}

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
    ) -> Result<Self, KernelError> {
        let expected = (width * height) as usize;
        let actual = weights.len();

        if expected != actual {
            Err(KernelError::DimensionMismatch { expected, actual })
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

    pub fn new_normalized(width: u32, height: u32, weights: Vec<f32>) -> Result<Self, KernelError> {
        let scaling_factor: f32 = {
            let sum: f32 = weights.iter().sum();
            if sum == 0.0 { 1.0 } else { 1.0 / sum }
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

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn scaling_factor(&self) -> f32 {
        self.scaling_factor
    }

    pub fn anchor_x(&self) -> u32 {
        self.anchor_x
    }

    pub fn anchor_y(&self) -> u32 {
        self.anchor_y
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_kernel_creations() {
        let cases = [
            (3, 3, 1, 1), // Square odd
            (3, 5, 1, 2), // Rectangle odd
            (6, 6, 3, 3), // Square even
            (6, 8, 3, 4), // Rectangle even
        ];

        for (w, h, ax, ay) in cases {
            let len = (w * h) as usize;
            let kernel = Kernel::new(1.0, w, h, vec![1.0; len]).unwrap();

            // We add the `"{w}x{h}"` context so we know which iteration panicked!
            assert_eq!(kernel.width(), w, "Width failed for {}x{}", w, h);
            assert_eq!(kernel.height(), h, "Height failed for {}x{}", w, h);
            assert_eq!(kernel.anchor_x(), ax, "Anchor X failed for {}x{}", w, h);
            assert_eq!(kernel.anchor_y(), ay, "Anchor Y failed for {}x{}", w, h);
        }
    }

    #[test]
    fn test_kernel_normalization() {
        let weights = vec![1.0, 2.0, 1.0, 2.0, 4.0, 2.0, 1.0, 2.0, 1.0];
        let kernel = Kernel::new_normalized(3, 3, weights).unwrap();

        // The sum of the weights is 16.0, so the scaling factor should be 1.0 / 16.0
        assert_eq!(kernel.scaling_factor(), 1.0 / 16.0);
    }

    #[test]
    fn test_kernel_dimension_mismatch() {
        let result = Kernel::new(1.0, 3, 3, vec![1.0; 4]);

        assert!(result.is_err());

        let err = result.unwrap_err();

        assert!(
            matches!(
                err,
                KernelError::DimensionMismatch {
                    expected: 9,
                    actual: 4
                }
            ),
            "Expected KernelError::DimensionMismatch, but got {:?}",
            err
        );
    }

    #[test]
    fn test_kernel_get_weight_out_of_bounds() {
        let kernel = Kernel::new(1.0, 3, 3, vec![1.0; 9]).unwrap();

        assert_eq!(kernel.get_weight(0, 0), Some(1.0));

        assert_eq!(kernel.get_weight(10, 10), None);
        assert_eq!(kernel.get_weight(-5, 0), None);
    }
}
