use std::fs;
use std::io::Write;

use clap::{Parser, Subcommand};

use stgn::utils::bytes_to_human;
use stgn::{Data, DataElement, Decoder};
use stgn::Encoder;

use image;

#[derive(Parser, Debug, Clone, PartialEq, Eq, Hash)]
struct EncyptSettings {
    #[arg(short='o', long="output", help = "Path to save the output image (for encryption) or decoded data (for decryption)")]
    output_file: Option<String>,

    #[arg(short='s', long, help = "Strings to encode into the image")]
    data_strings: Vec<String>,

    #[arg(short='f', long, help = "File paths to encode into the image")]
    data_files: Vec<String>,

    #[arg(short='c', long, help = "Whether to compress the data before encoding", default_value_t = true)]
    compress: bool,
}

#[derive(Parser, Debug, Clone, PartialEq, Eq, Hash)]
struct DecryptSettings {
    #[arg(short='e', long, help = "Export folder for decoded files (for decryption)", default_value = "decoded_output")]
    export_folder: String,

    #[arg(short='s', long, help = "File name for decoded strings (for decryption)", default_value = "decoded_strings.txt")]
    export_strings_file_name: String,
}

#[derive(Parser, Debug, Clone, PartialEq, Eq, Hash)]
struct MaxCapacitySettings {
    #[arg(short='b', long, help = "Show capacity in bytes instead of human readable format")]
    bytes: bool,
}

#[derive(Subcommand, Debug, Clone, PartialEq, Eq, Hash)]
enum Commands {
    Encode(EncyptSettings),
    Decode(DecryptSettings),
    MaxCapacity(MaxCapacitySettings)
}

#[derive(Parser, Debug, Clone, PartialEq, Eq, Hash)]
#[command(version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Commands,

    #[arg(short='f', long="file", help = "Path to the image file to encode into or decode from")]
    input_file: String,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    match args.command {
        Commands::Encode(enc_settings) => {
            let mut img = image::open(args.input_file.clone())?;
            if enc_settings.data_strings.is_empty() && enc_settings.data_files.is_empty() {
                eprintln!("No data provided to encode. Use --data-strings or --data-files.");
                return Ok(());
            }
            
            let mut data = Data::new();
            for s in enc_settings.data_strings {
                data.push(DataElement::text("string", &s));
            }
            for f in enc_settings.data_files {
                let content = fs::read(f)?;
                data.push(DataElement::bytes("file", content));
            }
            
            Encoder::encode_payload(&mut img, &data, None, enc_settings.compress)?;

            if let Err(e) = img.save(enc_settings.output_file.unwrap_or_else(|| "output.png".to_string())) {
                eprintln!("Failed to save image: {e}");
            } else {
                println!("Image saved as output.png");
            }
        },
        Commands::Decode(_dec_settings) => {
            println!("Decoding data from image...");
            let img = image::open(args.input_file.clone())?;
            let data = Decoder::decode_payload(&img, None)?;

            // create export folder if it doesn't exist
            fs::create_dir_all(&_dec_settings.export_folder)?;

            let strings = data.elements.iter().filter(|e| e.data_type == stgn::DataType::Text);

            let output_strings_file_path = std::path::Path::new(_dec_settings.export_folder.as_str()).join(_dec_settings.export_strings_file_name.as_str());

            let output_strings_file = std::fs::File::create(output_strings_file_path)?;
            let mut output_strings_writer = std::io::BufWriter::new(output_strings_file);

            if strings.clone().count() == 0 {
                println!("No text data found in the image.");
            } else {
                println!("Decoded text data:");
            }

            for elem in &data.elements {
                match elem.data_type {
                    stgn::DataType::Text => {
                        let s = std::str::from_utf8(&elem.value).unwrap_or("");
                        output_strings_writer.write(format!("{}: {}\n", elem.name, s).as_bytes())?;
                        println!("Exported text data to {}: {}", elem.name, s);
                    },
                    stgn::DataType::Binary => {
                        let file_path = std::path::Path::new(&_dec_settings.export_folder).join(&elem.name);
                        fs::write(&file_path, &elem.value)?;
                        println!("Exported binary data to {}", file_path.display());
                    }
                };
            }
        },
        Commands::MaxCapacity(max_capacity_settings) => {
            let img = image::open(args.input_file.clone())?;
            let capacity = Encoder::max_capacity(&img);
            let capacity_str = if max_capacity_settings.bytes {
                capacity.to_string() + " bytes"
            } else {
                bytes_to_human(capacity as u64)            
            };

            println!("Estimated capacity for hidden data: {}", capacity_str);
        }
    }
    Ok(())
}