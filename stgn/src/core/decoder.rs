use image::DynamicImage;
use postcard::from_bytes;

use crate::core::{
    auth::{EncryptionSecret, EncryptionType, SecureContext},
    data::Data,
    header::Header,
};

pub struct Decoder;

impl Decoder {
    /// Core: reads LSB bits, reconstructs auth+header, decrypts, deserializes into `Data`.
    pub fn decode_payload(
        img: &DynamicImage,
        secret: Option<&EncryptionSecret>,
    ) -> Result<Data, Box<dyn std::error::Error>> {
        let raw = Decoder::decode_raw(img, secret)?;
        let data: Data = from_bytes(&raw)?;
        Ok(data)
    }

    /// Low-level: reads LSB bits → raw decrypted bytes (the postcard-serialized Data).
    fn decode_raw(
        img: &DynamicImage,
        secret: Option<&EncryptionSecret>,
    ) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let channels = match img {
            DynamicImage::ImageRgb8(buf) => buf
                .pixels()
                .flat_map(|p| [p[0], p[1], p[2]])
                .collect::<Vec<_>>(),
            DynamicImage::ImageRgba8(buf) => buf
                .pixels()
                .flat_map(|p| [p[0], p[1], p[2]])
                .collect::<Vec<_>>(),
            _ => return Err("Unsupported image format".into()),
        };

        let mut bit_iter = channels.iter().map(|b| b & 1);

        macro_rules! read_byte {
            () => {{
                let mut byte = 0u8;
                for _ in 0..8 {
                    let bit = bit_iter.next().ok_or("Image too small")?;
                    byte = (byte << 1) | bit;
                }
                byte
            }};
        }

        // Auth block
        let auth_len = read_byte!() as usize;
        let mut auth_bytes = Vec::with_capacity(auth_len);
        for _ in 0..auth_len {
            auth_bytes.push(read_byte!());
        }
        let auth: SecureContext = from_bytes(&auth_bytes)?;

        // Header block
        let header_len = read_byte!() as usize;
        let mut header_bytes = Vec::with_capacity(header_len);
        for _ in 0..header_len {
            header_bytes.push(read_byte!());
        }
        let header: Header = from_bytes(&header_bytes)?;
        assert_eq!(header.magic, *crate::MAGIC, "Invalid magic number");

        // Payload
        let mut payload = Vec::with_capacity(header.length as usize);
        let mut byte = 0u8;
        let mut count = 0u8;
        for _ in 0..(header.length as usize * 8) {
            let bit = bit_iter.next().ok_or("Image ended early")?;
            byte = (byte << 1) | bit;
            count += 1;
            if count == 8 {
                payload.push(byte);
                byte = 0;
                count = 0;
            }
        }

        // Decrypt if needed
        let payload = if !matches!(auth.encryption_type, EncryptionType::None) {
            let s = secret.ok_or("Secret required for decryption")?;
            auth.decrypt(&payload, s)?
        } else {
            payload
        };

        // Decompressione solo se richiesto
        if header.compressed {
            use flate2::read::DeflateDecoder;
            use std::io::Read;
            let mut decoder = DeflateDecoder::new(&payload[..]);
            let mut decompressed = Vec::new();
            decoder.read_to_end(&mut decompressed)?;
            Ok(decompressed)
        } else {
            Ok(payload)
        }
    }

    // ── Convenience wrappers ──────────────────────────────────────────────────

    pub fn decode_string(
        img: &DynamicImage,
        secret: Option<&EncryptionSecret>,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let data = Decoder::decode_payload(img, secret)?;
        data.first_as_string()
            .ok_or_else(|| "No readable text entry found in payload".into())
    }

    pub fn decode_bytes(
        img: &DynamicImage,
        secret: Option<&EncryptionSecret>,
    ) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let data = Decoder::decode_payload(img, secret)?;
        data.first_bytes()
            .map(|b| b.to_vec())
            .ok_or_else(|| "No entry found in payload".into())
    }

    pub fn decode_file(
        img: &DynamicImage,
        output_path: &str,
        secret: Option<&EncryptionSecret>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let bytes = Decoder::decode_bytes(img, secret)?;
        std::fs::write(output_path, bytes)?;
        Ok(())
    }
}
