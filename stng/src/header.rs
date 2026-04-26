use serde::{Serialize, Deserialize};
use postcard::{from_bytes, to_slice};

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
pub struct Header {
    pub magic: [u8; 4], // "STNG"
    pub length: u32,     // Lunghezza dei dati nascosti in byte
}

impl Header {
    pub fn new(data_length: usize) -> Self {
        Header {
            magic: *crate::MAGIC,
            length: data_length as u32,
        }
    }
}