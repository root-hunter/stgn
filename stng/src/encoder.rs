use image::DynamicImage;

use crate::{auth::{EncryptionSecret, EncryptionType, SecureContext}, header::Header};

use postcard::to_slice;

pub struct Encoder;

impl Encoder {
    pub fn encode_secure(img: &mut DynamicImage, data: &[u8], secret: Option<&EncryptionSecret>) -> Result<(), Box<dyn std::error::Error>> {
        let mut data = data.to_vec();
        
        let auth = SecureContext::new(match secret {
            Some(EncryptionSecret::Xor(_)) => EncryptionType::Xor,
            Some(EncryptionSecret::Aes256(_)) => EncryptionType::Aes256,
            None => EncryptionType::None,
        });

        if secret.is_some() && !matches!(auth.encryption_type, EncryptionType::None) {
            data = auth.encrypt(&data, secret.unwrap());
        }
        
        let (width, height) = (img.width(), img.height());
        let capacity = (width * height * 3) as usize;

        let mut auth_buf = [0u8; 16];
        let auth_bytes = to_slice(&auth, &mut auth_buf).unwrap();
        let auth_len = auth_bytes.len() as u8;

        let header = Header::new(data.len());

        let mut header_buf = [0u8; 16];
        let header_bytes = to_slice(&header, &mut header_buf).unwrap();
        let header_len = header_bytes.len() as u8;

        let total_bits = (1 + auth_bytes.len() + header_bytes.len() + data.len()) * 8;
        if total_bits > capacity {
            return Err("Data too large".into());
        }

        let mut byte_iter = std::iter::once(&auth_len)
            .chain(auth_bytes.iter())
            .chain(std::iter::once(&header_len))
            .chain(header_bytes.iter())
            .chain(data.iter());
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
        secret: Option<&EncryptionSecret>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        Encoder::encode_secure(img, data.as_bytes(), secret)
    }

    pub fn encode_file(
        img: &mut DynamicImage,
        file_path: &str,
        secret: Option<&EncryptionSecret>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let data = std::fs::read(file_path)?;
        Encoder::encode_secure(img, &data, secret)
    }

    pub fn encode_bytes(
        img: &mut DynamicImage,
        data: &[u8],
        secret: Option<&EncryptionSecret>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        Encoder::encode_secure(img, data, secret)
    }

    pub fn max_capacity(img: &DynamicImage) -> usize {
        (img.width() * img.height() * 3 / 8) as usize
    }
}
