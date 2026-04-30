pub mod core;
pub mod embedding;
pub mod utils;

pub const MAGIC: &[u8; 4] = b"STGN";

pub use core::auth::{EncryptionSecret, EncryptionType};
pub use core::data::{Data, DataElement, DataType};
pub use core::decoder::Decoder;
pub use core::encoder::Encoder;
pub use embedding::pdf::PdfEmbedding;

mod tests {

    const IMAGE_TEST_PATH: &str = "tests/images/sample.jpg";
    const SECRET_TEST_PATH: &str = "tests/data/secret.txt";

    #[test]
    fn test_compression_roundtrip() {
        let mut img = ImageReader::open(asset(IMAGE_TEST_PATH))
            .unwrap()
            .decode()
            .unwrap();
        // Un payload ripetitivo che dovrebbe comprimersi bene
        let data = vec![42u8; 10_000];
        Encoder::encode_bytes(&mut img, &data, None, true).unwrap();
        let extracted_data = Decoder::decode_bytes(&img, None).unwrap();
        assert_eq!(data, extracted_data);
    }
    #[cfg(test)]
    use super::core::decoder::Decoder;
    #[cfg(test)]
    use super::core::encoder::Encoder;
    #[cfg(test)]
    use image::ImageReader;
    #[cfg(test)]
    use std::path::Path;

    /// Restituisce il path assoluto di una risorsa relativa alla root del workspace
    #[cfg(test)]
    fn asset(relative: &str) -> std::path::PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("workspace root")
            .join(relative)
    }

    #[test]
    fn test_steganography() {
        let mut img = ImageReader::open(asset(IMAGE_TEST_PATH))
            .unwrap()
            .decode()
            .unwrap();
        let data = "Ciao a tutti mi chiaddsa dsasd asd as dsa dsa adsa asdd d samo Antonio!!!!";

        Encoder::encode_string(&mut img, data, None, false).unwrap();

        let extracted_data = Decoder::decode_string(&img, None).unwrap();
        assert_eq!(data, extracted_data);
    }

    #[test]
    fn test_empty_string() {
        let mut img = ImageReader::open(asset(IMAGE_TEST_PATH))
            .unwrap()
            .decode()
            .unwrap();
        let data = "";

        Encoder::encode_string(&mut img, data, None, false).unwrap();

        let extracted_data = Decoder::decode_string(&img, None).unwrap();
        assert_eq!(data, extracted_data);
    }

    #[test]
    fn test_long_string() {
        let mut img = ImageReader::open(asset(IMAGE_TEST_PATH))
            .unwrap()
            .decode()
            .unwrap();
        let data = "Lorem ipsum dolor sit amet, consectetur adipiscing elit. Sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat. Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur. Excepteur sint occaecat cupidatat non proident, sunt in culpa qui officia deserunt mollit anim id est laborum.";

        Encoder::encode_string(&mut img, data, None, false).unwrap();

        let extracted_data = Decoder::decode_string(&img, None).unwrap();
        assert_eq!(data, extracted_data);
    }

    #[test]
    fn test_non_ascii_string() {
        let mut img = ImageReader::open(asset(IMAGE_TEST_PATH))
            .unwrap()
            .decode()
            .unwrap();
        let data = "Ciao a tutti mi chiaddsa dsasd asd as dsa dsa adsa asdd d samo Antonio!!!! こんにちは世界";

        Encoder::encode_string(&mut img, data, None, false).unwrap();
        let extracted_data = Decoder::decode_string(&img, None).unwrap();
        assert_eq!(data, extracted_data);
    }

    #[test]
    fn test_file_encoding() {
        let mut img = ImageReader::open(asset(IMAGE_TEST_PATH))
            .unwrap()
            .decode()
            .unwrap();
        let file_path = asset(SECRET_TEST_PATH);
        let file_path_str = file_path.to_str().unwrap();
        Encoder::encode_file(&mut img, file_path_str, None, false).unwrap();
        let extracted_data = Decoder::decode_string(&img, None).unwrap();
        let expected_data = std::fs::read_to_string(file_path_str).unwrap();
        assert_eq!(expected_data, extracted_data);
    }

    #[test]
    fn test_binary_encoding() {
        let mut img = ImageReader::open(asset(IMAGE_TEST_PATH))
            .unwrap()
            .decode()
            .unwrap();
        let data = vec![0, 255, 128, 64, 32, 16, 8, 4, 2, 1];
        Encoder::encode_bytes(&mut img, &data, None, false).unwrap();
        let extracted_data = Decoder::decode_bytes(&img, None).unwrap();
        assert_eq!(data, extracted_data);
    }

    #[test]
    fn test_magic_bytes() {
        let mut img = ImageReader::open(asset(IMAGE_TEST_PATH))
            .unwrap()
            .decode()
            .unwrap();
        let data = crate::MAGIC.to_vec();
        Encoder::encode_bytes(&mut img, &data, None, false).unwrap();
        let extracted_data = Decoder::decode_bytes(&img, None).unwrap();
        assert_eq!(data, extracted_data);
    }

    #[test]
    fn test_aes_encryption() {
        use crate::core::auth::EncryptionSecret;

        let mut img = ImageReader::open(asset(IMAGE_TEST_PATH))
            .unwrap()
            .decode()
            .unwrap();
        let data = "This is a secret message that will be encrypted using AES-256.";
        let secret = EncryptionSecret::Aes256(vec![0; 32]); // Chiave fittizia per il test
        Encoder::encode_string(&mut img, data, Some(&secret), false).unwrap();
        let extracted_data = Decoder::decode_string(&img, Some(&secret)).unwrap();
        assert_eq!(data, extracted_data);
    }

    #[test]
    fn test_multi_payload() {
        use crate::core::data::{Data, DataElement};

        let mut img = ImageReader::open(asset(IMAGE_TEST_PATH))
            .unwrap()
            .decode()
            .unwrap();

        let payload = Data::new()
            .add(DataElement::text("title", "Hello, world!"))
            .add(DataElement::text("note", "This is a second entry"))
            .add(DataElement::bytes("raw", vec![1, 2, 3, 4, 5]));

        Encoder::encode_payload(&mut img, &payload, None, false).unwrap();

        let decoded = Decoder::decode_payload(&img, None).unwrap();
        assert_eq!(decoded.get_text("title"), Some("Hello, world!"));
        assert_eq!(decoded.get_text("note"), Some("This is a second entry"));
        assert_eq!(decoded.get_bytes("raw"), Some([1u8, 2, 3, 4, 5].as_slice()));
    }
}
