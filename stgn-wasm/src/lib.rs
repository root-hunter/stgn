/// Create a ZIP archive containing the encoded image. Returns the ZIP as Vec<u8>.
#[wasm_bindgen]
pub fn zip_encoded_image(image_bytes: &[u8], filename: &str) -> Result<Vec<u8>, JsValue> {
    use std::io::Write;
    use zip::ZipWriter;
    use zip::write::FileOptions;
    let mut buf = Vec::new();
    let mut zip = ZipWriter::new(std::io::Cursor::new(&mut buf));
    let options = FileOptions::default().compression_method(zip::CompressionMethod::Deflated);
    zip.start_file(filename, options)
        .map_err(|e| JsValue::from_str(&format!("ZIP start_file error: {e}")))?;
    zip.write_all(image_bytes)
        .map_err(|e| JsValue::from_str(&format!("ZIP write error: {e}")))?;
    let cursor = zip
        .finish()
        .map_err(|e| JsValue::from_str(&format!("ZIP finish error: {e}")))?;
    let buf = cursor.into_inner();
    Ok(buf.clone())
}
#[wasm_bindgen]
pub fn encode_payload_size(
    entries_json: &str,
    encryption: &str,
    _key: &[u8],
    compress: bool,
) -> Result<usize, JsValue> {
    use flate2::{Compression, write::DeflateEncoder};
    use postcard::to_allocvec;
    use std::io::Write;
    use stgn::core::auth::EncryptionType;
    use stgn::core::header::Header;

    let entries: Vec<serde_json::Value> = serde_json::from_str(entries_json)
        .map_err(|e| JsValue::from_str(&format!("JSON parse error: {e}")))?;

    let mut data = Data::new();
    for entry in entries {
        let name = entry["name"].as_str().unwrap_or("entry").to_string();
        let typ = entry["type"].as_str().unwrap_or("text");
        let val = entry["value"].as_str().unwrap_or("");
        match typ {
            "binary" => {
                let bytes = B64
                    .decode(val)
                    .map_err(|e| JsValue::from_str(&format!("Base64 decode error: {e}")))?;
                data.push(DataElement::binary(name, bytes));
            }
            _ => {
                data.push(DataElement::text(name, val));
            }
        }
    }

    let serialized =
        to_allocvec(&data).map_err(|e| JsValue::from_str(&format!("Serialize error: {e}")))?;

    let payload_data = if compress {
        let mut encoder = DeflateEncoder::new(Vec::new(), Compression::default());
        encoder
            .write_all(&serialized)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
        encoder
            .finish()
            .map_err(|e| JsValue::from_str(&e.to_string()))?
    } else {
        serialized
    };

    let encryption_type = match encryption {
        "xor" => EncryptionType::Xor,
        "aes256" => EncryptionType::Aes256,
        _ => EncryptionType::None,
    };
    let auth = SecureContext::new(encryption_type);
    let mut auth_buf = [0u8; 16];
    let auth_bytes = postcard::to_slice(&auth, &mut auth_buf)
        .map_err(|e| JsValue::from_str(&format!("Auth serialize error: {e}")))?;
    let _auth_len = auth_bytes.len() as u8;

    let header = Header::new(payload_data.len(), compress);
    let mut header_buf = [0u8; 16];
    let header_bytes = postcard::to_slice(&header, &mut header_buf)
        .map_err(|e| JsValue::from_str(&format!("Header serialize error: {e}")))?;
    let _header_len = header_bytes.len() as u8;

    // Calcola dimensione effettiva (in byte)
    let total_bytes = 1 + auth_bytes.len() + 1 + header_bytes.len() + payload_data.len();
    Ok(total_bytes)
}
use base64::{Engine as _, engine::general_purpose::STANDARD as B64};
use stgn::core::auth::EncryptionSecret;
use stgn::core::auth::SecureContext;
use stgn::core::data::{Data, DataElement, DataType};
use stgn::core::decoder::Decoder;
use stgn::core::encoder::Encoder;
use wasm_bindgen::prelude::*;

fn parse_secret(encryption: &str, key: &[u8]) -> Option<EncryptionSecret> {
    match encryption {
        "xor" => Some(EncryptionSecret::Xor(key.to_vec())),
        "aes256" => {
            let mut k = [0u8; 32];
            let len = key.len().min(32);
            k[..len].copy_from_slice(&key[..len]);
            Some(EncryptionSecret::Aes256(k.to_vec()))
        }
        _ => None,
    }
}

fn img_to_png_bytes(img: image::DynamicImage) -> Result<Vec<u8>, JsValue> {
    let mut out: Vec<u8> = Vec::new();
    img.write_to(&mut std::io::Cursor::new(&mut out), image::ImageFormat::Png)
        .map_err(|e| JsValue::from_str(&e.to_string()))?;
    Ok(out)
}

// ── Legacy single-string API (kept for compatibility) ─────────────────────────

#[wasm_bindgen]
pub fn encode_string(image_bytes: &[u8], message: &str) -> Result<Vec<u8>, JsValue> {
    encode_string_secure(image_bytes, message, "none", &[], false)
}

#[wasm_bindgen]
pub fn encode_string_secure(
    image_bytes: &[u8],
    message: &str,
    encryption: &str,
    key: &[u8],
    compress: bool,
) -> Result<Vec<u8>, JsValue> {
    let mut img =
        image::load_from_memory(image_bytes).map_err(|e| JsValue::from_str(&e.to_string()))?;
    let secret = parse_secret(encryption, key);
    Encoder::encode_string(&mut img, message, secret.as_ref(), compress)
        .map_err(|e| JsValue::from_str(&e.to_string()))?;
    img_to_png_bytes(img)
}

#[wasm_bindgen]
pub fn encode_max_capacity(image_bytes: &[u8]) -> Result<usize, JsValue> {
    let img =
        image::load_from_memory(image_bytes).map_err(|e| JsValue::from_str(&e.to_string()))?;
    Ok(Encoder::max_capacity(&img))
}

#[wasm_bindgen]
pub fn decode_string(image_bytes: &[u8]) -> Result<String, JsValue> {
    decode_string_secure(image_bytes, "none", &[])
}

#[wasm_bindgen]
pub fn decode_string_secure(
    image_bytes: &[u8],
    encryption: &str,
    key: &[u8],
) -> Result<String, JsValue> {
    let img =
        image::load_from_memory(image_bytes).map_err(|e| JsValue::from_str(&e.to_string()))?;
    let secret = parse_secret(encryption, key);
    Decoder::decode_string(&img, secret.as_ref()).map_err(|e| JsValue::from_str(&e.to_string()))
}

// ── Multi-payload API ─────────────────────────────────────────────────────────
//
// JS <-> WASM exchange format (JSON array):
// [
//   { "name": "message", "type": "text",   "value": "plain text content" },
//   { "name": "file.png","type": "binary", "value": "<base64>" }
// ]

#[wasm_bindgen]
pub fn encode_payload(
    image_bytes: &[u8],
    entries_json: &str,
    encryption: &str,
    key: &[u8],
    compress: bool,
) -> Result<Vec<u8>, JsValue> {
    let entries: Vec<serde_json::Value> = serde_json::from_str(entries_json)
        .map_err(|e| JsValue::from_str(&format!("JSON parse error: {e}")))?;

    let mut data = Data::new();
    for entry in entries {
        let name = entry["name"].as_str().unwrap_or("entry").to_string();
        let typ = entry["type"].as_str().unwrap_or("text");
        let val = entry["value"].as_str().unwrap_or("");
        match typ {
            "binary" => {
                let bytes = B64
                    .decode(val)
                    .map_err(|e| JsValue::from_str(&format!("Base64 decode error: {e}")))?;
                data.push(DataElement::binary(name, bytes));
            }
            _ => {
                data.push(DataElement::text(name, val));
            }
        }
    }

    let mut img =
        image::load_from_memory(image_bytes).map_err(|e| JsValue::from_str(&e.to_string()))?;
    let secret = parse_secret(encryption, key);
    Encoder::encode_payload(&mut img, &data, secret.as_ref(), compress)
        .map_err(|e| JsValue::from_str(&e.to_string()))?;
    img_to_png_bytes(img)
}

#[wasm_bindgen]
pub fn decode_payload(image_bytes: &[u8], encryption: &str, key: &[u8]) -> Result<String, JsValue> {
    let img =
        image::load_from_memory(image_bytes).map_err(|e| JsValue::from_str(&e.to_string()))?;
    let secret = parse_secret(encryption, key);
    let data = Decoder::decode_payload(&img, secret.as_ref())
        .map_err(|e| JsValue::from_str(&e.to_string()))?;

    let mut arr = Vec::new();
    for elem in &data.elements {
        let typ = match elem.data_type {
            DataType::Text => "text",
            DataType::Binary => "binary",
        };
        let value = match elem.data_type {
            DataType::Text => std::str::from_utf8(&elem.value).unwrap_or("").to_string(),
            DataType::Binary => B64.encode(&elem.value),
        };
        arr.push(serde_json::json!({
            "name":  elem.name,
            "type":  typ,
            "value": value,
        }));
    }

    serde_json::to_string(&arr).map_err(|e| JsValue::from_str(&e.to_string()))
}
