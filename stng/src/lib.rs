pub mod decoder;
pub mod encoder;
pub mod utils;

mod tests {
    #[cfg(test)]
    use super::decoder::Decoder;
    #[cfg(test)]
    use super::encoder::Encoder;
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
        let mut img = ImageReader::open(asset("images/stego.jpg"))
            .unwrap()
            .decode()
            .unwrap();
        let data = "Ciao a tutti mi chiaddsa dsasd asd as dsa dsa adsa asdd d samo Antonio!!!!";

        Encoder::encode_string(&mut img, data).unwrap();

        let extracted_data = Decoder::decode_string(&img).unwrap();
        assert_eq!(data, extracted_data);
    }

    #[test]
    fn test_empty_string() {
        let mut img = ImageReader::open(asset("images/stego.jpg"))
            .unwrap()
            .decode()
            .unwrap();
        let data = "";

        Encoder::encode_string(&mut img, data).unwrap();

        let extracted_data = Decoder::decode_string(&img).unwrap();
        assert_eq!(data, extracted_data);
    }

    #[test]
    fn test_long_string() {
        let mut img = ImageReader::open(asset("images/stego.jpg"))
            .unwrap()
            .decode()
            .unwrap();
        let data = "Lorem ipsum dolor sit amet, consectetur adipiscing elit. Sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat. Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur. Excepteur sint occaecat cupidatat non proident, sunt in culpa qui officia deserunt mollit anim id est laborum.";

        Encoder::encode_string(&mut img, data).unwrap();

        let extracted_data = Decoder::decode_string(&img).unwrap();
        assert_eq!(data, extracted_data);
    }

    #[test]
    fn test_non_ascii_string() {
        let mut img = ImageReader::open(asset("images/stego.jpg"))
            .unwrap()
            .decode()
            .unwrap();
        let data = "Ciao a tutti mi chiaddsa dsasd asd as dsa dsa adsa asdd d samo Antonio!!!! こんにちは世界";

        Encoder::encode_string(&mut img, data).unwrap();
        let extracted_data = Decoder::decode_string(&img).unwrap();
        assert_eq!(data, extracted_data);
    }

    #[test]
    fn test_file_encoding() {
        let mut img = ImageReader::open(asset("images/dyno.png"))
            .unwrap()
            .decode()
            .unwrap();
        let file_path = asset("texts/commedia.txt");
        let file_path_str = file_path.to_str().unwrap();
        Encoder::encode_file(&mut img, file_path_str).unwrap();
        let extracted_data = Decoder::decode_string(&img).unwrap();
        let expected_data = std::fs::read_to_string(file_path_str).unwrap();
        assert_eq!(expected_data, extracted_data);
    }

    #[test]
    fn test_binary_encoding() {
        let mut img = ImageReader::open(asset("images/dyno.png"))
            .unwrap()
            .decode()
            .unwrap();
        let data = vec![0, 255, 128, 64, 32, 16, 8, 4, 2, 1];
        Encoder::encode_bytes(&mut img, &data).unwrap();
        let extracted_data = Decoder::decode_bytes(&img).unwrap();
        assert_eq!(data, extracted_data);
    }
}
