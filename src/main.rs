use std::{
    fs::{self, File},
    io::{BufRead, BufReader, Read},
};

#[derive(Debug, Clone, Copy)]
struct Pixel {
    r: u8,
    g: u8,
    b: u8,
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
struct Image {
    width: u32,
    height: u32,
    pixels: Vec<Pixel>,
}

impl Image {
    fn from_file_path(path: String) -> Result<Self, &'static str> {
        //TODO: skip comments in the header
        let f = File::open(path).map_err(|_| "Failed while opening the file")?;

        let mut width: u32 = 0;
        let mut height: u32 = 0;

        let mut parser_stt = PpmParserStt::MagicNumber;
        let mut magic_number = PpmMagicNumber::P3;

        let reader = BufReader::new(f);
        let mut bytes_iter = reader.bytes();

        let mut next_word = || -> Result<String, &'static str> {
            let mut word = String::new();

            // skip leading whitespace
            for byte in &mut bytes_iter {
                let byte = byte.map_err(|_| "IO error reading byte")?;
                if !byte.is_ascii_whitespace() {
                    word.push(byte as char);
                    break;
                }
            }

            // read until next whitespace
            for byte in &mut bytes_iter {
                let byte = byte.map_err(|_| "IO error reading byte")?;
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
                        _ => return Err("Unsupported magic number found"),
                    }
                    parser_stt.advance();
                }
                PpmParserStt::Width => {
                    width = word.parse().map_err(|_| "Error parsing width")?;
                    parser_stt.advance();
                }
                PpmParserStt::Height => {
                    height = word.parse().map_err(|_| "Error parsing height")?;
                    parser_stt.advance();
                }
                PpmParserStt::MaxColor => {
                    if word != "255" {
                        return Err("Only 8-bit depth (255) is supported");
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

        let mut crr_pixel_idx: usize = 0;

        if magic_number == PpmMagicNumber::P6 {
            // P6 BINARY PARSING
            // You can just loop over `bytes_iter` here. Every 3 bytes = 1 Pixel!
            // Example: let r = bytes_iter.next().unwrap().unwrap();
        } else {
            // P3 parsing
            let mut current_r = 0;
            let mut current_g = 0;

            while let Ok(value) = next_word() {
                if value.is_empty() {
                    // EOF!
                    break;
                }

                let value = value
                    .parse::<u8>()
                    .map_err(|_| "Error while trying to parse pixel color value")?;
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
            }
        }

        Ok(img)
    }

    #[inline]
    fn get_pixel(&self, x: u32, y: u32) -> Option<Pixel> {
        if x >= self.width || y >= self.height {
            None
        } else {
            Some(self.pixels[(y * self.width + x) as usize])
        }
    }

    #[inline]
    fn get_pixel_mut(&mut self, x: u32, y: u32) -> Option<&mut Pixel> {
        if x >= self.width || y >= self.height {
            None
        } else {
            Some(&mut self.pixels[(y * self.width + x) as usize])
        }
    }
}

fn main() {
    dbg!(
        "{}",
        Image::from_file_path("example_images/pallete.ppm".into()).unwrap()
    );
}
