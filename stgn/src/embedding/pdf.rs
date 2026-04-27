use image::{DynamicImage, GenericImageView};

use lopdf::content::{Content, Operation};
use lopdf::dictionary;
use lopdf::{Document, Object, Stream};

pub struct PdfEmbedding;

impl PdfEmbedding {
    pub fn embed(img: DynamicImage) -> Result<Vec<u8>, String> {
        let mut doc = Document::with_version("1.5");
        let pages_id = doc.new_object_id();

        let (img_width, img_height) = img.dimensions();
        let raw_pixels = img.to_rgb8().into_raw();

        let image_stream = Stream::new(
            dictionary! {
                "Type"             => "XObject",
                "Subtype"          => "Image",
                "Width"            => img_width as i64,
                "Height"           => img_height as i64,
                "ColorSpace"       => "DeviceRGB",
                "BitsPerComponent" => 8,
            },
            raw_pixels,
        );
        let image_id = doc.add_object(image_stream);

        let font_id = doc.add_object(dictionary! {
            "Type"     => "Font",
            "Subtype"  => "Type1",
            "BaseFont" => "Courier",
        });

        let resources_id = doc.add_object(dictionary! {
            "Font" => dictionary! {
                "F1" => font_id,
            },
            "XObject" => dictionary! {
                "STGNImage1" => image_id,
            },
        });

        let draw_width: i64 = 300;
        let draw_height: i64 = 200;
        let x: i64 = 100;
        let y: i64 = 400;

        let content = Content {
            operations: vec![
                Operation::new("q", vec![]),
                Operation::new(
                    "cm",
                    vec![
                        draw_width.into(),
                        0.into(),
                        0.into(),
                        draw_height.into(),
                        x.into(),
                        y.into(),
                    ],
                ),
                Operation::new("Do", vec!["STGNImage1".into()]),
                Operation::new("Q", vec![]),
            ],
        };

        let content_id = doc.add_object(Stream::new(dictionary! {}, content.encode().unwrap()));

        let page_id = doc.add_object(dictionary! {
            "Type"     => "Page",
            "Parent"   => pages_id,
            "Contents" => content_id,
        });

        let pages = dictionary! {
            "Type"      => "Pages",
            "Kids"      => vec![page_id.into()],
            "Count"     => 1,
            "Resources" => resources_id,
            "MediaBox"  => vec![0.into(), 0.into(), 595.into(), 842.into()],
        };
        doc.objects.insert(pages_id, Object::Dictionary(pages));

        let catalog_id = doc.add_object(dictionary! {
            "Type"  => "Catalog",
            "Pages" => pages_id,
        });

        doc.trailer.set("Root", catalog_id);
        doc.compress();

        let mut pdf_bytes = Vec::new();
        doc.save_to(&mut pdf_bytes).unwrap();

        println!("First 100 bytes of PDF: {:?}", &pdf_bytes[..100]);

        Ok(pdf_bytes)
    }

    pub fn extract(pdf_bytes: &[u8]) -> Result<DynamicImage, String> {
        let mut doc =
            Document::load_mem(pdf_bytes).map_err(|e| format!("Errore caricamento PDF: {}", e))?;

        let pages_id = doc
            .catalog()
            .map_err(|e| format!("{}", e))?
            .get(b"Pages")
            .and_then(|o| o.as_reference())
            .map_err(|e| format!("Pages non trovato: {}", e))?;

        let pages_dict = doc
            .get_object(pages_id)
            .and_then(|o| o.as_dict())
            .map_err(|e| format!("Pages dict non trovato: {}", e))?
            .clone();

        let page_id = pages_dict
            .get(b"Kids")
            .and_then(|o| o.as_array())
            .map_err(|e| format!("Kids non trovato: {}", e))?
            .first()
            .ok_or("Nessuna pagina")?
            .as_reference()
            .map_err(|e| format!("Page ref non valida: {}", e))?;

        let page_dict = doc
            .get_object(page_id)
            .and_then(|o| o.as_dict())
            .map_err(|e| format!("Page dict non trovato: {}", e))?
            .clone();

        let resources = page_dict
            .get(b"Resources")
            .ok()
            .or_else(|| pages_dict.get(b"Resources").ok())
            .ok_or("Resources non trovato")?;

        let resources_dict = match resources {
            Object::Dictionary(d) => d.clone(),
            Object::Reference(r) => doc
                .get_object(*r)
                .and_then(|o| o.as_dict())
                .map_err(|e| format!("Resources ref: {}", e))?
                .clone(),
            _ => return Err("Resources tipo inatteso".to_string()),
        };

        let xobject_obj = resources_dict
            .get(b"XObject")
            .map_err(|e| format!("XObject non trovato: {}", e))?;

        let xobjects = match xobject_obj {
            Object::Dictionary(d) => d.clone(),
            Object::Reference(r) => doc
                .get_object(*r)
                .and_then(|o| o.as_dict())
                .map_err(|e| format!("XObject ref: {}", e))?
                .clone(),
            _ => return Err("XObject tipo inatteso".to_string()),
        };

        let image_ref = xobjects
            .get(b"STGNImage1")
            .and_then(|o| o.as_reference())
            .map_err(|e| format!("STGNImage1 non trovato: {}", e))?;

        let image_stream = doc
            .get_object_mut(image_ref)
            .and_then(|o| o.as_stream_mut())
            .map_err(|e| format!("Stream non trovato: {}", e))?;

        image_stream
            .decompress()
            .map_err(|e| format!("Decompressione fallita: {}", e))?;

        let width = image_stream
            .dict
            .get(b"Width")
            .and_then(|o| Ok(o.as_i64().ok()))
            .unwrap_or(None)
            .ok_or("Width mancante")? as u32;

        let height = image_stream
            .dict
            .get(b"Height")
            .and_then(|o| Ok(o.as_i64().ok()))
            .unwrap_or(None)
            .ok_or("Height mancante")? as u32;

        let raw_pixels = image_stream.content.clone();

        let expected_len = (width * height * 3) as usize;
        if raw_pixels.len() != expected_len {
            return Err(format!(
                "Dimensione pixel non corrisponde: attesi {} bytes, trovati {}",
                expected_len,
                raw_pixels.len()
            ));
        }

        let img_buffer = image::RgbImage::from_raw(width, height, raw_pixels)
            .ok_or("Impossibile creare RgbImage dai pixel grezzi")?;

        Ok(DynamicImage::ImageRgb8(img_buffer))
    }
}
