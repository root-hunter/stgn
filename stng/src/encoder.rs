use image::DynamicImage;

pub struct Encoder;

impl Encoder {
    pub fn encode(img: &mut DynamicImage, data: &[u8]) -> Result<(), Box<dyn std::error::Error>> {
        let (width, height) = (img.width(), img.height());
        let capacity = (width * height * 3) as usize;

        let header = (data.len() as u32).to_be_bytes();

        let total_bits = (header.len() + data.len()) * 8;
        if total_bits > capacity {
            return Err("Data too large".into());
        }

        let mut byte_iter = header.iter().chain(data.iter());
        let mut current = 0u8;
        let mut bit_idx = 8;

        let mut next_bit = || -> Option<u8> {
            if bit_idx == 8 {
                current = *byte_iter.next()?;
                bit_idx = 0;
            }
            let bit = (current >> (7 - bit_idx)) & 1;
            bit_idx += 1;
            Some(bit)
        };

        match img {
            DynamicImage::ImageRgb8(buf) => {
                for pixel in buf.pixels_mut() {
                    for c in 0..3 {
                        if let Some(bit) = next_bit() {
                            pixel[c] = (pixel[c] & 0xFE) | bit;
                        } else {
                            return Ok(());
                        }
                    }
                }
            }

            DynamicImage::ImageRgba8(buf) => {
                for pixel in buf.pixels_mut() {
                    for c in 0..3 {
                        if let Some(bit) = next_bit() {
                            pixel[c] = (pixel[c] & 0xFE) | bit;
                        } else {
                            return Ok(());
                        }
                    }
                }
            }

            _ => return Err("Unsupported format".into()),
        }

        Ok(())
    }

    pub fn encode_string(
        img: &mut DynamicImage,
        data: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        Encoder::encode(img, data.as_bytes())
    }

    pub fn encode_file(
        img: &mut DynamicImage,
        file_path: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let data = std::fs::read(file_path)?;
        Encoder::encode(img, &data)
    }

    pub fn encode_bytes(
        img: &mut DynamicImage,
        data: &[u8],
    ) -> Result<(), Box<dyn std::error::Error>> {
        Encoder::encode(img, data)
    }

    pub fn max_capacity(img: &DynamicImage) -> usize {
        (img.width() * img.height() * 3 / 8) as usize
    }
}
