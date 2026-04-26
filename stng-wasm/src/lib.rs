use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn encode(message: &str, image_bytes: &[u8]) -> Vec<u8> {
    // TODO: wrappa le funzioni di stng::encoder
    todo!()
}

#[wasm_bindgen]
pub fn decode(image_bytes: &[u8]) -> String {
    // TODO: wrappa le funzioni di stng::decoder
    todo!()
}
