use image::{DynamicImage, GenericImageView};

use crate::{HEADER_SIZE, utils::bytes_to_human};

pub fn decode(img: &DynamicImage) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let width = img.width();

    let mut extracted_data_binary = String::new();
    let mut data_length_binary = String::new();

    while data_length_binary.len() < HEADER_SIZE {
        let pixel_index = data_length_binary.len() / 3;
        let x = (pixel_index % width as usize) as u32;
        let y = (pixel_index / width as usize) as u32;

        let pixel = img.get_pixel(x, y);

        for j in 0..3 {
            if data_length_binary.len() >= HEADER_SIZE {
                break;
            }

            let bit = pixel[j] & 1;
            data_length_binary.push_str(&format!("{}", bit));
        }
    }

    let data_length = u32::from_str_radix(&data_length_binary, 2).unwrap();

    println!("Extracted data length: {}", data_length);

    let total_bits = HEADER_SIZE + data_length as usize * 8;
    let mut i = HEADER_SIZE; // Start after the header
    while i < total_bits {
        let pixel_index = i / 3;
        let channel = i % 3;
        let x = (pixel_index % width as usize) as u32;
        let y = (pixel_index / width as usize) as u32;

        let pixel = img.get_pixel(x, y);
        let bit = pixel[channel] & 1;
        extracted_data_binary.push_str(&format!("{}", bit));

        i += 1;
    }

    assert!(extracted_data_binary.len() / 8 == data_length as usize, "Extracted data length does not match the expected length");

    println!("Bytes extracted: {}", bytes_to_human(extracted_data_binary.len() as u64 / 8));

    let extracted_data_bytes = extracted_data_binary
        .as_bytes()
        .chunks(8)
        .map(|chunk| {
            let byte_str = std::str::from_utf8(chunk).unwrap();
            u8::from_str_radix(byte_str, 2).unwrap()
        })
        .collect::<Vec<u8>>();

    Ok(extracted_data_bytes)
}

pub fn decode_string(img: &DynamicImage) -> Result<String, Box<dyn std::error::Error>> {
    let extracted_data_bytes = decode(img)?;
    let extracted_data = String::from_utf8(extracted_data_bytes)?;
    Ok(extracted_data)
}

pub fn decode_file(img: &DynamicImage, output_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let extracted_data_bytes = decode(img)?;
    std::fs::write(output_path, extracted_data_bytes)?;

    Ok(())
}