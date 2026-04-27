use image::ImageReader;
use stgn::{
    core::decoder::Decoder,
    core::encoder::Encoder,
    utils::{bytes_to_human, init_logging},
};
use tracing::info;

use image::GenericImageView;
use lopdf::content::{Content, Operation};
use lopdf::dictionary;
use lopdf::{Document, Object, Stream};

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

    Encoder::encode_file(&mut img, file_path, None, false)?;
    img.save("images/encoded_image.png")?;

    let img2 = ImageReader::open("images/encoded_image.png")?.decode()?;

    let output_path = "texts/output.txt";
    Decoder::decode_file(&img2, output_path, None)?;

    let mut doc = Document::with_version("1.5");
    let pages_id = doc.new_object_id();

    // --- Carica immagine e converti in pixel RGB grezzi (non compressi) ---
    let (img_width, img_height) = img.dimensions();
    let raw_pixels = img.to_rgb8().into_raw(); // Vec<u8> di byte RGB grezzi

    // Crea l'XObject immagine SENZA Filter (nessuna compressione)
    let image_stream = Stream::new(
        dictionary! {
            "Type"             => "XObject",
            "Subtype"          => "Image",
            "Width"            => img_width as i64,
            "Height"           => img_height as i64,
            "ColorSpace"       => "DeviceRGB",
            "BitsPerComponent" => 8,
            // Nessun "Filter" = pixel grezzi, immagine originale non compressa
        },
        raw_pixels,
    );
    let image_id = doc.add_object(image_stream);

    // Font (necessario anche se non si scrive testo)
    let font_id = doc.add_object(dictionary! {
        "Type"     => "Font",
        "Subtype"  => "Type1",
        "BaseFont" => "Courier",
    });

    // Dizionario risorse: registra font e immagine
    let resources_id = doc.add_object(dictionary! {
        "Font" => dictionary! {
            "F1" => font_id,
        },
        "XObject" => dictionary! {
            "Im1" => image_id, // "Im1" è il nome usato nel content stream
        },
    });

    // --- Parametri di posizione e dimensione dell'immagine nella pagina ---
    // La pagina è 595 x 842 pt (formato A4)
    // In PDF, Y=0 è in basso a sinistra
    let draw_width: i64 = 300; // larghezza visualizzata in punti PDF
    let draw_height: i64 = 200; // altezza visualizzata in punti PDF
    let x: i64 = 100; // posizione X (da sinistra)
    let y: i64 = 400; // posizione Y (dal basso)

    // Content stream: disegna l'immagine con la matrice di trasformazione
    // cm = [scaleX 0 0 scaleY translateX translateY]
    // Do = disegna il XObject con il nome specificato
    let content = Content {
        operations: vec![
            Operation::new("q", vec![]), // salva stato grafico
            Operation::new(
                "cm",
                vec![
                    draw_width.into(), // scaleX  → larghezza
                    0.into(),
                    0.into(),
                    draw_height.into(), // scaleY  → altezza
                    x.into(),           // translateX
                    y.into(),           // translateY
                ],
            ),
            Operation::new("Do", vec!["Im1".into()]), // disegna immagine
            Operation::new("Q", vec![]),              // ripristina stato grafico
        ],
    };

    let content_id = doc.add_object(Stream::new(dictionary! {}, content.encode().unwrap()));

    // Pagina
    let page_id = doc.add_object(dictionary! {
        "Type"     => "Page",
        "Parent"   => pages_id,
        "Contents" => content_id,
    });

    // Albero pagine (root)
    let pages = dictionary! {
        "Type"      => "Pages",
        "Kids"      => vec![page_id.into()],
        "Count"     => 1,
        "Resources" => resources_id,
        "MediaBox"  => vec![0.into(), 0.into(), 595.into(), 842.into()],
    };
    doc.objects.insert(pages_id, Object::Dictionary(pages));

    // Catalogo documento
    let catalog_id = doc.add_object(dictionary! {
        "Type"  => "Catalog",
        "Pages" => pages_id,
    });

    doc.trailer.set("Root", catalog_id);
    doc.compress();
    doc.save("output_with_image.pdf").unwrap();

    println!("PDF salvato: output_with_image.pdf");

    Ok(())
}
