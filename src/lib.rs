pub mod decoder;
pub mod encoder;
pub mod utils;

mod tests {
    #[cfg(test)]
    use image::ImageReader;
    #[cfg(test)]
    use super::encoder::Encoder;
    #[cfg(test)]
    use super::decoder::Decoder;

    #[test]
    fn test_steganography() {
        let mut img = ImageReader::open("images/stego.jpg")
            .unwrap()
            .decode()
            .unwrap();
        let data = "Ciao a tutti mi chiaddsa dsasd asd as dsa dsa adsa asdd d samo Antonio!!!!";

        let img = Encoder::encode_string(&mut img, data).unwrap();

        let extracted_data = Decoder::decode_string(&img).unwrap();
        assert_eq!(data, extracted_data);
    }

    #[test]
    fn test_empty_string() {
        let mut img = ImageReader::open("images/stego.jpg")
            .unwrap()
            .decode()
            .unwrap();
        let data = "";

        let img = Encoder::encode_string(&mut img, data).unwrap();

        let extracted_data = Decoder::decode_string(&img).unwrap();
        assert_eq!(data, extracted_data);
    }

    #[test]
    fn test_long_string() {
        let mut img = ImageReader::open("images/stego.jpg")
            .unwrap()
            .decode()
            .unwrap();
        let data = "Lorem ipsum dolor sit amet, consectetur adipiscing elit. Sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat. Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur. Excepteur sint occaecat cupidatat non proident, sunt in culpa qui officia deserunt mollit anim id est laborum.";

        let img = Encoder::encode_string(&mut img, data).unwrap();

        let extracted_data = Decoder::decode_string(&img).unwrap();
        assert_eq!(data, extracted_data);
    }

    #[test]
    fn test_non_ascii_string() {
        let mut img = ImageReader::open("images/stego.jpg")
            .unwrap()
            .decode()
            .unwrap();
        let data = "Ciao a tutti mi chiaddsa dsasd asd as dsa dsa adsa asdd d samo Antonio!!!! こんにちは世界";

        let img = Encoder::encode_string(&mut img, data).unwrap();
        let extracted_data = Decoder::decode_string(&img).unwrap();
        assert_eq!(data, extracted_data);
    }
}
