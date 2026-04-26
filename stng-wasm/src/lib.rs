use stng::auth::EncryptionSecret;
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

#[wasm_bindgen]
pub fn encode_string(image_bytes: &[u8], message: &str) -> Result<Vec<u8>, JsValue> {
    encode_string_secure(image_bytes, message, "none", &[])
}

#[wasm_bindgen]
pub fn encode_string_secure(
    image_bytes: &[u8],
    message: &str,
    encryption: &str,
    key: &[u8],
) -> Result<Vec<u8>, JsValue> {
    let mut img =
        image::load_from_memory(image_bytes).map_err(|e| JsValue::from_str(&e.to_string()))?;

    let secret = parse_secret(encryption, key);

    stng::encoder::Encoder::encode_string(&mut img, message, secret.as_ref())
        .map_err(|e| JsValue::from_str(&e.to_string()))?;

    let mut out: Vec<u8> = Vec::new();
    img.write_to(&mut std::io::Cursor::new(&mut out), image::ImageFormat::Png)
        .map_err(|e| JsValue::from_str(&e.to_string()))?;

    Ok(out)
}

#[wasm_bindgen]
pub fn encode_max_capacity(image_bytes: &[u8]) -> Result<usize, JsValue> {
    let img =
        image::load_from_memory(image_bytes).map_err(|e| JsValue::from_str(&e.to_string()))?;

    Ok(stng::encoder::Encoder::max_capacity(&img))
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

    stng::decoder::Decoder::decode_string(&img, secret.as_ref())
        .map_err(|e| JsValue::from_str(&e.to_string()))
}
