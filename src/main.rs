use image::{GenericImage, GenericImageView, ImageReader};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut img = ImageReader::open("images/stego.jpg")?.decode()?;

    let data = "This is a secret message hidden in the image!";
    let data_bytes = data.as_bytes();
    let data_length = data_bytes.len() as u32;

    let data_length_bytes = data_length.to_be_bytes();

    let mut data_binary = data_length_bytes.iter()
        .map(|byte| format!("{:08b}", byte))
        .collect::<Vec<String>>()
        .join("");

    println!("Data length in bytes: {}", data_length);
    println!("Data length in binary: {}", data_binary);

    data_binary.push_str(&data_bytes.iter()
        .map(|byte| format!("{:08b}", byte))
        .collect::<Vec<String>>()
        .join(""));
    
    let width = img.width();
    let height = img.height();
    let pixels_count = width * height;

    println!("Image dimensions: {}x{}", width, height);
    println!("Total number of pixels: {}", pixels_count);

    let mut x = 0;
    let mut y = 0;

    while x < width && y < height && !data_binary.is_empty() {
        let pixel = img.get_pixel(x, y);
        let mut r = pixel[0];
        let mut g = pixel[1];
        let mut b = pixel[2];
        let a = pixel[3];

        for j in 0..3 {
            if data_binary.is_empty() {
                break;
            }

            let bit = data_binary.chars().next().unwrap().to_digit(2).unwrap() as u8;

            if bit == 0 {
                match j {
                    0 => r = (r & 0xFE) | 0,
                    1 => g = (g & 0xFE) | 0,
                    2 => b = (b & 0xFE) | 0,
                    _ => unreachable!(),
                }
            } else {
                match j {
                    0 => r = (r & 0xFE) | 1,
                    1 => g = (g & 0xFE) | 1,
                    2 => b = (b & 0xFE) | 1,
                    _ => unreachable!(),
                }
            }

            data_binary.remove(0);
        }

        img.put_pixel(x, y, image::Rgba([r, g, b, a]));

        x += 1;
        if x >= width {
            x = 0;
            y += 1;
        }
    }

    img.save("images/encoded_image.png")?;

    // decode
    let img = ImageReader::open("images/encoded_image.png")?.decode()?;
    let mut extracted_data_binary = String::new();

    // Read data length from the first 32 bits
    let mut data_length_binary = String::new();
    while data_length_binary.len() < 32 {
        let pixel = img.get_pixel((data_length_binary.len() / 3) as u32, (data_length_binary.len() / 3 / width as usize) as u32);

        for j in 0..3 {
            if data_length_binary.len() >= 32 {
                break;
            }

            let bit = pixel[j] & 1;
            data_length_binary.push_str(&format!("{}", bit));
        }
    }

    println!("Extracted data length binary: {}", data_length_binary);
    println!("Extracted data length binary (first 32 bits): {}", &data_length_binary);

    let data_length = u32::from_str_radix(&data_length_binary, 2).unwrap();
    println!("Extracted data length: {}", data_length);

    let mut i = 0; // Start after the first 32 bits which represent the data length
    while i < (data_length as usize * 8) + 32 {
        let x = (i / 3) as u32;
        let y = (i / 3) as u32 / width;

        if x <= 10 {
            i += 3; // Skip the first 11 pixels (33 bits) which contain the data length
            continue; // Skip the first 11 pixels which contain the data length
        }

        let pixel = img.get_pixel(x, y);

        for j in 0..3 {
            if i >= (data_length as usize * 8) + 32 {
                break;
            }

            let bit = pixel[j] & 1;
            extracted_data_binary.push_str(&format!("{}", bit));

            i += 1;
        }
    }

    println!("extracted_data_binary length: {}", extracted_data_binary.len());
    println!("Extracted binary data: {}", extracted_data_binary);
    let extracted_data_bytes = extracted_data_binary
        .as_bytes()
        .chunks(8)
        .map(|chunk| {
            let byte_str = std::str::from_utf8(chunk).unwrap();
            u8::from_str_radix(byte_str, 2).unwrap()
        })
        .collect::<Vec<u8>>();

    let extracted_data = String::from_utf8(extracted_data_bytes).unwrap();
    println!("Extracted data: {}", extracted_data);
    Ok(())
}