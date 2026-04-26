use image::ImageReader;
use stng::{decoder::{decode_file, decode_string}, encoder::{encode_file, encode_string}};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut img = ImageReader::open("images/stego.jpg")?.decode()?;

    let file_path = "texts/test.txt";

    encode_file(&mut img, file_path)?;

    let output_path = "texts/output.txt";
    decode_file(&img, output_path)?;

    Ok(())
}