use image::DynamicImage;
use tracing::info;

pub struct Decoder;

impl Decoder {
    pub fn decode(img: &DynamicImage) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let mut channels: Box<dyn Iterator<Item = u8>> = match img {
            DynamicImage::ImageRgb8(buf) => Box::new(buf.pixels().flat_map(|p| [p[0], p[1], p[2]])),
            DynamicImage::ImageRgba8(buf) => {
                Box::new(buf.pixels().flat_map(|p| [p[0], p[1], p[2]]))
            }
            _ => return Err("Unsupported format".into()),
        };

        // ---- HEADER (32 bit) ----
        let mut header: u32 = 0;

        for _ in 0..32 {
            let byte = channels.next().ok_or("Image too small (header)")?;
            header = (header << 1) | (byte & 1) as u32;
        }

        let total_bits = header as usize * 8;

        info!("Decoding {} bytes from image...", header);

        let mut out = Vec::with_capacity(header as usize);

        let mut byte = 0u8;
        let mut count = 0;

        for _i in 0..total_bits {
            let bit = channels.next().ok_or("Image ended early (data)")? & 1;

            byte = (byte << 1) | bit;
            count += 1;

            if count == 8 {
                out.push(byte);
                byte = 0;
                count = 0;
            }
        }

        Ok(out)
    }

    pub fn decode_string(img: &DynamicImage) -> Result<String, Box<dyn std::error::Error>> {
        let extracted_data_bytes = Decoder::decode(img)?;
        let extracted_data = String::from_utf8(extracted_data_bytes)?;
        Ok(extracted_data)
    }

    pub fn decode_file(
        img: &DynamicImage,
        output_path: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let extracted_data_bytes = Decoder::decode(img)?;
        std::fs::write(output_path, extracted_data_bytes)?;

        Ok(())
    }

    pub fn decode_bytes(img: &DynamicImage) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        Decoder::decode(img)
    }
}
