use std::{
    fs::File,
    io::{BufReader, BufWriter, Read, Write},
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
}

#[derive(Debug, Clone, Copy)]
pub struct Pixel {
    r: u8,
    g: u8,
    b: u8,
}

impl Pixel {
    pub fn black() -> Self {
        Pixel { r: 0, g: 0, b: 0 }
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

#[derive(Debug, Clone)]
pub struct Image {
    width: u32,
    height: u32,
    pixels: Vec<Pixel>,
}

impl Image {
    pub fn empty(width: u32, height: u32) -> Self {
        Image {
            width,
            height,
            pixels: vec![Pixel { r: 0, g: 0, b: 0 }; (width * height) as usize],
        }
    }

    pub fn from_file_path(path: String) -> Result<Self, ImageError> {
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
    pub fn save_to_file(&self, path: String) -> Result<(), ImageError> {
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

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }
}
