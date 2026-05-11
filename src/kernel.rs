use std::fs;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum KernelError {
    #[error("Dimension mismatch: expected {expected} weights (width * height), but got {actual}")]
    DimensionMismatch { expected: usize, actual: usize },

    #[error("IO Error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Could not parse line: ")]
    BadLine(String),

    #[error("Failed to parse kernel scaling factor: {0}")]
    ParseScalingFactor(#[source] std::num::ParseFloatError),

    #[error("Failed to parse kernel {field}. The value '{value}' is invalid.")]
    ParseDimensionValue {
        field: String,
        value: String,
        #[source]
        source: std::num::ParseIntError,
    },

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
    pub fn from_file_path(path: String, normalize: bool) -> Result<Self, KernelError> {
        let content = fs::read_to_string(path)?;

        let mut valid_lines = content
            .lines()
            .map(|l| l.trim())
            .filter(|l| !l.is_empty() && !l.starts_with('#'));

        let sf_line = valid_lines
            .next()
            .ok_or_else(|| KernelError::BadLine("Missing scaling factor".into()))?;
        let scaling_factor = sf_line
            .parse::<f32>()
            .map_err(KernelError::ParseScalingFactor)?;

        let dim_line = valid_lines
            .next()
            .ok_or_else(|| KernelError::BadLine("Missing dimensions".into()))?;

        let mut dim_parts = dim_line.split_whitespace();
        let height_str = dim_parts
            .next()
            .ok_or_else(|| KernelError::BadLine("Missing height".into()))?;
        let width_str = dim_parts
            .next()
            .ok_or_else(|| KernelError::BadLine("Missing width".into()))?;

        let height = height_str
            .parse::<u32>()
            .map_err(|e| KernelError::ParseDimensionValue {
                field: "height".into(),
                value: height_str.into(),
                source: e,
            })?;

        let width = width_str
            .parse::<u32>()
            .map_err(|e| KernelError::ParseDimensionValue {
                field: "width".into(),
                value: width_str.into(),
                source: e,
            })?;

        let mut weights = Vec::with_capacity((width * height) as usize);

        for line in valid_lines {
            let parts: Vec<&str> = line.split_whitespace().collect();

            if parts.len() != width as usize {
                return Err(KernelError::BadLine(format!(
                    "Expected {} values in weight line, got {}",
                    width,
                    parts.len()
                )));
            }

            for w_str in parts {
                let w = w_str.parse::<f32>().map_err(KernelError::ParseWeight)?;
                weights.push(w);
            }
        }

        if normalize {
            Self::new_normalized(width, height, weights)
        } else {
            Self::new(scaling_factor, width, height, weights)
        }
    }

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

    fn get_test_kernel_path(filename: &str) -> String {
        let mut path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push("test_data/kernels");
        path.push(filename);
        path.to_str()
            .expect("Path contains invalid unicode")
            .to_string()
    }

    #[test]
    fn test_kernel_from_file_path_with_comments_and_spaces() {
        let path = get_test_kernel_path("messy_kernel.kbildr");

        let kernel = Kernel::from_file_path(path, false).unwrap();

        assert_eq!(kernel.scaling_factor(), 0.5);
        assert_eq!(kernel.width(), 2);
        assert_eq!(kernel.height(), 2);

        assert_eq!(kernel.get_weight(-1, -1), Some(1.0));
        assert_eq!(kernel.get_weight(0, 0), Some(4.0));
    }

    #[test]
    fn test_kernel_from_file_path_missing_dimensions() {
        let path = get_test_kernel_path("missing_dimensions.kbildr");

        let result = Kernel::from_file_path(path, false);

        assert!(result.is_err());

        let err = result.unwrap_err();
        assert!(
            matches!(&err, KernelError::BadLine(msg) if msg.contains("Missing width")),
            "Expected a BadLine error regarding missing width, got: {:?}",
            err
        );
    }

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
