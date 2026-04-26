use image::DynamicImage;

pub fn encode(
    img: &mut image::DynamicImage,
    data: &[u8],
) -> Result<DynamicImage, Box<dyn std::error::Error>> {
    let width = img.width();
    let height = img.height();
    let pixels_count = width * height;

    let data_length = data.len() as u32;
    let header = data_length.to_be_bytes();

    let total_bits = (header.len() + data.len()) * 8;

    assert!(total_bits <= pixels_count as usize * 3, "Data too large");

    // Iteratore sui byte (header + data)
    let mut bytes_iter = header.iter().chain(data.iter());

    let mut current_byte = 0u8;
    let mut bit_index = 8; // forza load iniziale

    let mut next_bit = || -> Option<u8> {
        if bit_index == 8 {
            current_byte = *bytes_iter.next()?;
            bit_index = 0;
        }

        let bit = (current_byte >> (7 - bit_index)) & 1;
        bit_index += 1;
        Some(bit)
    };

    match img {
        DynamicImage::ImageRgb8(buf) => {
            for pixel in buf.pixels_mut() {
                for channel in 0..3 {
                    if let Some(bit) = next_bit() {
                        pixel[channel] = (pixel[channel] & 0xFE) | bit;
                    } else {
                        break;
                    }
                }
            }
        }
        DynamicImage::ImageRgba8(buf) => {
            for pixel in buf.pixels_mut() {
                for channel in 0..3 {
                    if let Some(bit) = next_bit() {
                        pixel[channel] = (pixel[channel] & 0xFE) | bit;
                    } else {
                        break;
                    }
                }
            }
        }
        _ => panic!("Unsupported image format"),
    };

    Ok(img.clone())
}

pub fn encode_string(img: &mut DynamicImage, data: &str) -> Result<DynamicImage, Box<dyn std::error::Error>> {
    encode(img, data.as_bytes())
}

pub fn encode_file(
    img: &mut DynamicImage,
    file_path: &str,
) -> Result<DynamicImage, Box<dyn std::error::Error>> {
    let data = std::fs::read(file_path)?;
    encode(img, &data)
}
