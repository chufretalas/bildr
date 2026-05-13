use std::{
    fs::File,
    io::{BufReader, BufWriter, Read, Write},
    path::Path,
};

use thiserror::Error;

#[derive(Debug, Error)]
pub enum ImageError {
    #[error("IO Error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Failed to parse the {field}. The value '{value}' is invalid.")]
    ParseInteger {
        field: String,
        value: String,
        #[source]
        source: std::num::ParseIntError,
    },

    #[error("Unsupported magic number found: {0}")]
    UnsupportedMagicNumber(String),

    #[error("Only 8-bit depth (255) is supported. Found: {0}")]
    UnsupportedDepth(String),

    #[error("The position asked for is out of bound")]
    OutOfBounds,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Pixel {
    r: u8,
    g: u8,
    b: u8,
}

impl Pixel {
    pub fn black() -> Self {
        Self::from_rgb(0, 0, 0)
    }

    pub fn from_rgb(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    pub fn r(&self) -> u8 {
        return self.r;
    }

    pub fn g(&self) -> u8 {
        return self.g;
    }

    pub fn b(&self) -> u8 {
        return self.b;
    }

    pub fn set_rgb(&mut self, r: u8, g: u8, b: u8) {
        self.r = r;
        self.g = g;
        self.b = b;
    }

    pub fn to_minifb_pixel(&self) -> u32 {
        let r = self.r() as u32;
        let g = self.g() as u32;
        let b = self.b() as u32;

        (r << 16) | (g << 8) | b
    }

    pub fn from_minifb_pixel(minifb_pixel: u32) -> Self {
        Pixel::from_rgb(
            (minifb_pixel >> 16) as u8, // red
            (minifb_pixel >> 8) as u8,  // green
            minifb_pixel as u8,         // blue
        )
    }
}

#[derive(PartialEq, Eq)]
enum PpmMagicNumber {
    P3,
    P6,
}

#[derive(PartialEq, Eq)]
enum PixelColor {
    Red,
    Green,
    Blue,
}

#[derive(PartialEq, Eq)]
enum PpmParserStt {
    MagicNumber,
    Width,
    Height,
    MaxColor,
    Pixels(PixelColor),
}

impl PpmParserStt {
    fn advance(&mut self) {
        *self = match self {
            Self::MagicNumber => Self::Width,
            Self::Width => Self::Height,
            Self::Height => Self::MaxColor,
            Self::MaxColor => Self::Pixels(PixelColor::Red),
            Self::Pixels(PixelColor::Red) => Self::Pixels(PixelColor::Green),
            Self::Pixels(PixelColor::Green) => Self::Pixels(PixelColor::Blue),
            Self::Pixels(PixelColor::Blue) => Self::Pixels(PixelColor::Red),
        };
    }
}

// TODO: change width and height into usize
#[derive(Debug, Clone)]
pub struct Image {
    width: u32,
    height: u32,
    pixels: Vec<Pixel>,
}

impl Image {
    pub fn black(width: u32, height: u32) -> Self {
        Self::solid(width, height, Pixel::black())
    }

    pub fn solid(width: u32, height: u32, color: Pixel) -> Self {
        Image {
            width,
            height,
            pixels: vec![color; (width * height) as usize],
        }
    }

    pub fn from_minifb_buffer(minifb_buffer: Vec<u32>, width: u32, height: u32) -> Self {
        Image {
            width,
            height,
            pixels: minifb_buffer
                .into_iter()
                .map(Pixel::from_minifb_pixel)
                .collect(),
        }
    }

    pub fn from_file_path(path: &Path) -> Result<Self, ImageError> {
        //TODO: skip comments in the header
        let file = File::open(path)?;

        let mut width: u32 = 0;
        let mut height: u32 = 0;

        let mut parser_stt = PpmParserStt::MagicNumber;
        let mut magic_number = PpmMagicNumber::P3;

        let reader = BufReader::new(file);
        let mut bytes_iter = reader.bytes();

        let mut next_word = || -> Result<String, ImageError> {
            let mut word = String::with_capacity(3);

            // skip leading whitespace
            for byte in &mut bytes_iter {
                let byte = byte?;
                if !byte.is_ascii_whitespace() {
                    word.push(byte as char);
                    break;
                }
            }

            // read until next whitespace
            for byte in &mut bytes_iter {
                let byte = byte?;
                if byte.is_ascii_whitespace() {
                    break;
                }
                word.push(byte as char);
            }

            Ok(word)
        };

        while parser_stt != PpmParserStt::Pixels(PixelColor::Red)
            && parser_stt != PpmParserStt::Pixels(PixelColor::Green)
            && parser_stt != PpmParserStt::Pixels(PixelColor::Blue)
        {
            let word = next_word()?;

            match parser_stt {
                PpmParserStt::MagicNumber => {
                    match word.as_str() {
                        "P3" => magic_number = PpmMagicNumber::P3,
                        "P6" => magic_number = PpmMagicNumber::P6,
                        _ => return Err(ImageError::UnsupportedMagicNumber(word)),
                    }
                    parser_stt.advance();
                }
                PpmParserStt::Width => {
                    width = word.parse().map_err(|e| ImageError::ParseInteger {
                        field: "width".to_string(),
                        value: word.clone(), // We clone because `word` might be moved/used elsewhere
                        source: e,
                    })?;
                    parser_stt.advance();
                }
                PpmParserStt::Height => {
                    height = word.parse().map_err(|e| ImageError::ParseInteger {
                        field: "height".to_string(),
                        value: word.clone(),
                        source: e,
                    })?;
                    parser_stt.advance();
                }
                PpmParserStt::MaxColor => {
                    if word != "255" {
                        return Err(ImageError::UnsupportedDepth(word));
                    }
                    parser_stt.advance();
                }
                PpmParserStt::Pixels(_) => unreachable!(),
            }
        }

        // pre-allocate the pixels vector
        let mut img = Image {
            width: width,
            height: height,
            pixels: Vec::with_capacity((width * height) as usize),
        };

        let mut current_r = 0;
        let mut current_g = 0;

        let mut save_pixel = |value: u8| {
            match parser_stt {
                PpmParserStt::Pixels(PixelColor::Red) => current_r = value,
                PpmParserStt::Pixels(PixelColor::Green) => current_g = value,
                PpmParserStt::Pixels(PixelColor::Blue) => {
                    img.pixels.push(Pixel {
                        r: current_r,
                        g: current_g,
                        b: value,
                    });
                }
                _ => unreachable!(),
            }
            parser_stt.advance();
        };

        match magic_number {
            PpmMagicNumber::P3 => {
                while let Ok(value) = next_word() {
                    if value.is_empty() {
                        // EOF!
                        break;
                    }

                    let value = value.parse::<u8>().map_err(|e| ImageError::ParseInteger {
                        field: "pixel".to_string(),
                        value: value.clone(),
                        source: e,
                    })?;

                    save_pixel(value);
                }
            }
            PpmMagicNumber::P6 => {
                while let Some(Ok(value)) = bytes_iter.next() {
                    save_pixel(value);
                }
            }
        }

        Ok(img)
    }

    ///Saves the image to the desired location as a P6 .ppm file
    pub fn save_to_file(&self, path: &Path) -> Result<(), ImageError> {
        let file = File::create(path)?;
        let mut writer = BufWriter::new(file);

        writeln!(writer, "P6")?;
        writeln!(writer, "{} {}", self.width, self.height)?;
        writeln!(writer, "255")?;

        let mut flat_pixels = Vec::with_capacity(self.pixels.len() * 3);

        self.pixels.iter().for_each(|p| {
            flat_pixels.push(p.r);
            flat_pixels.push(p.g);
            flat_pixels.push(p.b);
        });

        writer.write_all(&flat_pixels)?;

        Ok(())
    }

    #[inline]
    pub fn get_pixel(&self, x: i32, y: i32) -> Option<Pixel> {
        if x < 0 || x >= self.width as i32 || y < 0 || y >= self.height as i32 {
            None
        } else {
            Some(self.pixels[(y * self.width as i32 + x) as usize])
        }
    }

    #[inline]
    pub fn get_pixel_mut(&mut self, x: i32, y: i32) -> Option<&mut Pixel> {
        if x < 0 || x >= self.width as i32 || y < 0 || y >= self.height as i32 {
            None
        } else {
            Some(&mut self.pixels[(y * self.width as i32 + x) as usize])
        }
    }

    pub fn set_pixel(&mut self, x: i32, y: i32, pixel: Pixel) -> Result<(), ImageError> {
        if x < 0 || x >= self.width as i32 || y < 0 || y >= self.height as i32 {
            Err(ImageError::OutOfBounds)
        } else {
            self.pixels[(y * self.width as i32 + x) as usize] = pixel;
            Ok(())
        }
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn to_minifb_buffer(&self) -> Vec<u32> {
        self.pixels.iter().map(Pixel::to_minifb_pixel).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use std::fs;
    use std::path::PathBuf;

    fn get_test_image_path(filename: &str) -> PathBuf {
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push("test_data/imgs");
        path.push(filename);
        path
    }

    #[test]
    fn test_from_file_path_p3() {
        let img = Image::from_file_path(&get_test_image_path("6x3_p3.ppm")).unwrap();

        assert_eq!(img.width(), 3, "Loaded image's width does not match!");
        assert_eq!(img.height(), 2, "Loaded image's height does not match!");
        assert_eq!(
            img.pixels,
            vec![
                Pixel::from_rgb(255, 0, 0),
                Pixel::from_rgb(0, 255, 0),
                Pixel::from_rgb(0, 0, 255),
                Pixel::from_rgb(255, 255, 0),
                Pixel::from_rgb(0, 255, 255),
                Pixel::from_rgb(255, 0, 255)
            ],
            "Loaded image's pixel data does not match!"
        );
    }

    #[test]
    fn test_from_file_path_p6() {
        let img = Image::from_file_path(&get_test_image_path("2x2_p6.ppm")).unwrap();

        assert_eq!(img.width(), 2, "Loaded image's width does not match!");
        assert_eq!(img.height(), 2, "Loaded image's height does not match!");
        assert_eq!(
            img.pixels,
            vec![
                Pixel::from_rgb(255, 0, 0),
                Pixel::from_rgb(0, 0, 0),
                Pixel::from_rgb(0, 0, 0),
                Pixel::from_rgb(0, 255, 0)
            ],
            "Loaded image's pixel data does not match!"
        );
    }

    #[test]
    fn test_save_to_file_matches_golden_reference() {
        let mut img = Image::black(2, 2);
        img.get_pixel_mut(0, 0).unwrap().set_rgb(255, 0, 0);
        img.get_pixel_mut(1, 1).unwrap().set_rgb(0, 255, 0);

        let reference_path = get_test_image_path("2x2_p6.ppm");

        let temp_path = env::temp_dir().join(format!("test_out_{}.ppm", std::process::id()));

        img.save_to_file(&temp_path)
            .expect("Failed to save temp file");

        let generated_bytes = fs::read(&temp_path).expect("Failed to read generated file");
        let reference_bytes = fs::read(&reference_path).expect("Failed to read reference file");

        assert_eq!(
            generated_bytes, reference_bytes,
            "The generated image bytes do not match the golden reference!"
        );

        let _ = fs::remove_file(temp_path);
    }

    #[test]
    fn test_image_parsing_unsupported_magic_number() {
        let path = get_test_image_path("bad_magic.ppm");
        let result = Image::from_file_path(&path);

        assert!(result.is_err());
        assert!(
            matches!(result.unwrap_err(), ImageError::UnsupportedMagicNumber(_)),
            "Should have failed with UnsupportedMagicNumber"
        );
    }

    #[test]
    fn test_image_error_io_not_found() {
        let path = get_test_image_path("non_existent_file.ppm");
        let result = Image::from_file_path(&path);

        assert!(result.is_err());
        assert!(
            matches!(result.unwrap_err(), ImageError::Io(_)),
            "Should have failed with an Io error for a missing file"
        );
    }

    #[test]
    fn test_image_error_parse_integer() {
        let path = get_test_image_path("bad_width.ppm");
        let result = Image::from_file_path(&path);

        assert!(result.is_err());

        let err = result.unwrap_err();

        assert!(
            matches!(&err, ImageError::ParseInteger { field, .. } if field == "width"),
            "Should have failed parsing the width, but got: {:?}",
            err
        );
    }

    #[test]
    fn test_image_error_unsupported_depth() {
        let path = get_test_image_path("bad_depth.ppm");
        let result = Image::from_file_path(&path);

        assert!(result.is_err());

        let err = result.unwrap_err();
        assert!(
            matches!(&err, ImageError::UnsupportedDepth(val) if val == "65535"),
            "Should have failed with UnsupportedDepth for '65535', but got: {:?}",
            err
        );
    }
}
