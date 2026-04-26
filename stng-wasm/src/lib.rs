use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn encode_string(image_bytes: &[u8], message: &str) -> Result<Vec<u8>, JsValue> {
    let mut img =
        image::load_from_memory(image_bytes).map_err(|e| JsValue::from_str(&e.to_string()))?;

    stng::encoder::Encoder::encode_string(&mut img, message, None)
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
    let img =
        image::load_from_memory(image_bytes).map_err(|e| JsValue::from_str(&e.to_string()))?;

    stng::decoder::Decoder::decode_string(&img).map_err(|e| JsValue::from_str(&e.to_string()))
}
