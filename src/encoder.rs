use image::DynamicImage;
use tracing::{info, debug, error};

pub struct Encoder;

impl Encoder {
    pub fn encode(
        img: &mut DynamicImage,
        data: &[u8],
    ) -> Result<DynamicImage, Box<dyn std::error::Error>> {
        let (width, height) = (img.width(), img.height());
        let capacity = (width * height * 3) as usize;

        let header = (data.len() as u32).to_be_bytes();

        let total_bits = (header.len() + data.len()) * 8;
        assert!(total_bits <= capacity, "Data too large");

        info!("Encoding {} bytes into image...", data.len());
        let mut bits = Vec::with_capacity(total_bits);

        for byte in header.iter().chain(data.iter()) {
            for i in (0..8).rev() {
                bits.push((byte >> i) & 1);
            }
        }

        let mut bit_idx = 0;

        match img {
            DynamicImage::ImageRgb8(buf) => {
                for pixel in buf.pixels_mut() {
                    for c in 0..3 {
                        if bit_idx >= bits.len() {
                            return Ok(img.clone());
                        }
                        pixel[c] = (pixel[c] & 0xFE) | bits[bit_idx];
                        bit_idx += 1;
                    }
                }
            }

            DynamicImage::ImageRgba8(buf) => {
                for pixel in buf.pixels_mut() {
                    for c in 0..3 {
                        if bit_idx >= bits.len() {
                            return Ok(img.clone());
                        }
                        pixel[c] = (pixel[c] & 0xFE) | bits[bit_idx];
                        bit_idx += 1;
                    }
                }
            }

            _ => return Err("Unsupported format".into()),
        }

        Ok(img.clone())
    }

    pub fn encode_string(
        img: &mut DynamicImage,
        data: &str,
    ) -> Result<DynamicImage, Box<dyn std::error::Error>> {
        Encoder::encode(img, data.as_bytes())
    }

    pub fn encode_file(
        img: &mut DynamicImage,
        file_path: &str,
    ) -> Result<DynamicImage, Box<dyn std::error::Error>> {
        let data = std::fs::read(file_path)?;
        Encoder::encode(img, &data)
    }

    pub fn encode_bytes(
        img: &mut DynamicImage,
        data: &[u8],
    ) -> Result<DynamicImage, Box<dyn std::error::Error>> {
        Encoder::encode(img, data)
    }
}
