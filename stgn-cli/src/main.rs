use std::io::Write;
use std::{fs, vec};

use clap::{Parser, Subcommand};

use stgn::Encoder;
use stgn::utils::bytes_to_human;
use stgn::{Data, DataElement, Decoder, EncryptionSecret};

use image;

#[derive(clap::ValueEnum, Parser, Debug, Clone, PartialEq, Eq, Hash)]
enum EncryptionType {
    None,
    Aes256,
}

#[derive(Parser, Debug, Clone, PartialEq, Eq, Hash)]
struct EncyptSettings {
    #[arg(
        short = 'o',
        long = "output",
        help = "Path to save the output image (for encryption) or decoded data (for decryption)"
    )]
    output: Option<String>,

    #[arg(short = 's', help = "Strings to encode into the image")]
    strings: Vec<String>,

    #[arg(short = 'f', help = "File paths to encode into the image")]
    files: Vec<String>,

    #[arg(
        short = 'c',
        help = "Whether to compress the data before encoding",
        default_value_t = true
    )]
    compress: bool,
}

#[derive(Parser, Debug, Clone, PartialEq, Eq, Hash)]
struct DecryptSettings {
    #[arg(
        short = 'e',
        help = "Export folder for decoded files (for decryption)",
        default_value = "decoded_output"
    )]
    export_folder: String,

    #[arg(
        short = 's',
        help = "File name for decoded strings (for decryption)",
        default_value = "decoded_strings.txt"
    )]
    export_strings_file_name: String,
}

#[derive(Parser, Debug, Clone, PartialEq, Eq, Hash)]
struct MaxCapacitySettings {
    #[arg(
        short = 'b',
        help = "Show capacity in bytes instead of human readable format"
    )]
    bytes: bool,
}

#[derive(Subcommand, Debug, Clone, PartialEq, Eq, Hash)]
enum Commands {
    #[command(about = "Encode data into an image")]
    Encode(EncyptSettings),
    
    #[command(about = "Decode data from an image")]
    Decode(DecryptSettings),

    #[command(about = "Show the maximum capacity of an image")]
    MaxCapacity(MaxCapacitySettings),
}

#[derive(Parser, Debug, Clone, PartialEq, Eq, Hash)]
#[command(version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Commands,

    #[arg(
        short = 'i',
        help = "Path to the image file to encode into or decode from"
    )]
    input: String,

    #[arg(short='e', help = "Encryption method to use (for encryption)", value_enum, default_value_t = EncryptionType::None)]
    encryption: EncryptionType,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let encryption = match &args.encryption {
        EncryptionType::None => stgn::core::auth::EncryptionType::None,
        EncryptionType::Aes256 => stgn::core::auth::EncryptionType::Aes256,
    };

    let encryption_secret = if encryption != stgn::core::auth::EncryptionType::None {
        let password = rpassword::prompt_password("Enter encryption key: ")?;
        let password_confirm = rpassword::prompt_password("Confirm encryption key: ")?;

        if password != password_confirm {
            eprintln!("Encryption keys do not match.");
            return Ok(());
        }

        let mut password_bytes = vec![0u8; 32];

        let secret = match encryption {
            stgn::core::auth::EncryptionType::Aes256 => {
                if password_bytes.len() < 32 {
                    // Pad the password to 32 bytes if it's too short
                    password_bytes[..password.len()].copy_from_slice(password.as_bytes());
                } else if password_bytes.len() > 32 {
                    // Truncate the password to 32 bytes if it's too long
                    password_bytes = password_bytes[..32].to_vec();
                }
                EncryptionSecret::Aes256(password_bytes.to_vec())
            }
            _ => unreachable!(),
        };

        Some(secret)
    } else {
        None
    };

    let secret = if let Some(ref secret) = encryption_secret {
        Some(secret)
    } else {
        None
    };

    match args.command {
        Commands::Encode(enc_settings) => {
            let mut img = image::open(args.input.clone())?;
            if enc_settings.strings.is_empty() && enc_settings.files.is_empty() {
                eprintln!("No data provided to encode. Use --data-strings or --data-files.");
                return Ok(());
            }

            let mut data = Data::new();

            for (i, s) in enc_settings.strings.iter().enumerate() {
                data.push(DataElement::text(&format!("string_{}", i + 1), s));
            }
            for f in enc_settings.files {
                let content = fs::read(&f)?;
                let file_name = std::path::Path::new(&f)
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("file");
                data.push(DataElement::bytes(file_name, content));
            }

            Encoder::encode_payload(&mut img, &data, secret, enc_settings.compress)?;

            if let Err(e) = img.save(
                enc_settings
                    .output
                    .unwrap_or_else(|| "output.png".to_string()),
            ) {
                eprintln!("Failed to save image: {e}");
            } else {
                println!("Image saved as output.png");
            }
        }
        Commands::Decode(_dec_settings) => {
            println!("Decoding data from image...");
            let img = image::open(args.input.clone())?;
            let data = Decoder::decode_payload(&img, secret)?;

            // create export folder if it doesn't exist
            fs::create_dir_all(&_dec_settings.export_folder)?;

            let strings = data
                .elements
                .iter()
                .filter(|e| e.data_type == stgn::DataType::Text);

            let output_strings_file_path =
                std::path::Path::new(_dec_settings.export_folder.as_str())
                    .join(_dec_settings.export_strings_file_name.as_str());

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
                        output_strings_writer
                            .write(format!("{}: {}\n", elem.name, s).as_bytes())?;
                        println!("Exported text data to {}: {}", elem.name, s);
                    }
                    stgn::DataType::Binary => {
                        let file_path =
                            std::path::Path::new(&_dec_settings.export_folder).join(&elem.name);
                        fs::write(&file_path, &elem.value)?;
                        println!("Exported binary data to {}", file_path.display());
                    }
                };
            }
        }
        Commands::MaxCapacity(max_capacity_settings) => {
            let img = image::open(args.input.clone())?;
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
