use image::ImageReader;
use stgn::{
    core::{decoder::Decoder, encoder::Encoder},
    embedding::pdf::PdfEmbedding,
    utils::{bytes_to_human, init_logging},
};
use tracing::info;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    init_logging();

    let image_path = "images/dyno.png";

    let mut img = ImageReader::open(image_path)?.decode()?;

    info!("Encoding file into image...");

    let width = img.width();
    let height = img.height();
    info!("Image path: {}", image_path);
    info!("Image format: {:?}", img.color());
    info!("Image dimensions: {}x{}", width, height);
    info!(
        "Image encoding capacity: {}",
        bytes_to_human(((width * height * 3) / 8 - 4).into())
    ); // -4 for header

    let file_path = "texts/commedia.txt";

    let encoder = Encoder::default();

    encoder.encode_file(&mut img, file_path, None)?;
    img.save("images/encoded_image.png")?;

    let img2 = ImageReader::open("images/encoded_image.png")?.decode()?;

    let output_path = "texts/output.txt";
    Decoder::decode_file(&img2, output_path, None)?;

    let _pdf_bytes = PdfEmbedding::embed(img)?;

    std::fs::write("output.pdf", &_pdf_bytes)?;

    let decoded_img = PdfEmbedding::extract(&_pdf_bytes)?;
    decoded_img.save("images/decoded_from_pdf.png")?;

    let img2 = ImageReader::open("images/decoded_from_pdf.png")?.decode()?;
    Decoder::decode_file(&img2, "texts/decoded_from_pdf.txt", None)?;

    Ok(())
}
