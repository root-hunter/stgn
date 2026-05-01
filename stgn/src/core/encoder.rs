use flate2::{Compression, write::DeflateEncoder};
use image::{DynamicImage, GenericImageView};
use postcard::{to_allocvec, to_slice};

use crate::core::{
    auth::{EncryptionSecret, EncryptionType, SecureContext},
    data::{Data, DataElement},
    header::Header,
};

pub const CHANNEL_R: u8 = 0x01;
pub const CHANNEL_G: u8 = 0x02;
pub const CHANNEL_B: u8 = 0x04;

pub type RGBChannelMask = u8; // Bitmask: 1 for R, 2 for G, 4 for B

pub const BIT_MASK_LOW: u8 = 0x01; // Use only the least significant bit
pub const BIT_MASK_2: u8 = 0x03; // Use the two least significant bits
pub const BIT_MASK_4: u8 = 0x0F; // Use the four least significant bits
pub const BIT_MASK_8: u8 = 0xFF; // Use all bits
pub const BIT_MASK_HIGH: u8 = 0x80; // Use the most significant bit

pub type BitMask = u8;

pub struct EncoderConfig {
    pub channels: RGBChannelMask,
    pub bit_mask: BitMask,
    pub compress: bool,
}

impl Default for EncoderConfig {
    fn default() -> Self {
        Self {
            channels: CHANNEL_R | CHANNEL_G | CHANNEL_B, // Use all RGB channels by default (1 | 2 | 4)
            bit_mask: BIT_MASK_LOW,                      // Use only the least significant bit
            compress: true,
        }
    }
}

pub struct Encoder {
    pub configs: EncoderConfig,
}

impl Default for Encoder {
    fn default() -> Self {
        Self {
            configs: EncoderConfig::default(),
        }
    }
}

impl Encoder {
    pub fn new(configs: EncoderConfig) -> Self {
        Self { configs }
    }

    /// Core: serializes `data` with postcard, optionally encrypts, then LSB-encodes.
    pub fn encode_payload(
        &self,
        img: &mut DynamicImage,
        data: &Data,
        secret: Option<&EncryptionSecret>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let serialized = to_allocvec(data)?;
        self.encode_raw(img, &serialized, secret)
    }

    /// Low-level: takes already-assembled raw bytes, wraps them in auth+header, writes LSB.
    fn encode_raw(
        &self,
        img: &mut DynamicImage,
        raw: &[u8],
        secret: Option<&EncryptionSecret>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let auth = SecureContext::new(match secret {
            Some(EncryptionSecret::Aes256(_)) => EncryptionType::Aes256,
            _ => EncryptionType::None,
        });

        // Comprimi il payload solo se richiesto
        let payload_data = if self.configs.compress {
            let mut encoder = DeflateEncoder::new(Vec::new(), Compression::default());
            use std::io::Write;
            encoder.write_all(raw)?;
            encoder.finish()?
        } else {
            raw.to_vec()
        };

        let payload = if secret.is_some() && !matches!(auth.encryption_type, EncryptionType::None) {
            auth.encrypt(&payload_data, secret.unwrap())?
        } else {
            payload_data
        };

        let (width, height) = img.dimensions();
        let capacity = (width * height * 3) as usize;

        let mut auth_buf = [0u8; 16];
        let auth_bytes = to_slice(&auth, &mut auth_buf)?;
        let auth_len = auth_bytes.len() as u8;

        let header = Header::new(payload.len(), self.configs.compress);
        let mut header_buf = [0u8; 16];
        let header_bytes = to_slice(&header, &mut header_buf)?;
        let header_len = header_bytes.len() as u8;

        let total_bits = (1 + auth_bytes.len() + 1 + header_bytes.len() + payload.len()) * 8;
        if total_bits > capacity {
            return Err("Data too large for this image".into());
        }

        let mut byte_iter = std::iter::once(&auth_len)
            .chain(auth_bytes.iter())
            .chain(std::iter::once(&header_len))
            .chain(header_bytes.iter())
            .chain(payload.iter());

        let mut current = 0u8;
        let mut bit_idx = 8u8;

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
            _ => return Err("Unsupported image format".into()),
        }

        Ok(())
    }

    // ── Convenience wrappers ──────────────────────────────────────────────────

    pub fn encode_string(
        &self,
        img: &mut DynamicImage,
        text: &str,
        secret: Option<&EncryptionSecret>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.encode_payload(img, &Data::from_text(text), secret)
    }

    pub fn encode_file(
        &self,
        img: &mut DynamicImage,
        file_path: &str,
        secret: Option<&EncryptionSecret>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let content = std::fs::read(file_path)?;
        let name = std::path::Path::new(file_path)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("file")
            .to_string();
        self.encode_payload(img, &Data::from_file(name, content), secret)
    }

    pub fn encode_bytes(
        &self,
        img: &mut DynamicImage,
        data: &[u8],
        secret: Option<&EncryptionSecret>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.encode_payload(img, &Data::from_bytes_payload(data.to_vec()), secret)
    }

    /// Encode multiple named entries in a single image.
    pub fn encode_multi(
        &self,
        img: &mut DynamicImage,
        entries: Vec<DataElement>,
        secret: Option<&EncryptionSecret>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut payload = Data::new();
        for e in entries {
            payload.push(e);
        }
        self.encode_payload(img, &payload, secret)
    }

    pub fn max_capacity(&self, img: &DynamicImage) -> usize {
        let (width, height) = img.dimensions();
        (width * height * 3 / 8) as usize
    }
}
