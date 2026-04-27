use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
pub struct Header {
    pub magic: [u8; 4],   // "STGN"
    pub length: u32,      // Lunghezza dei dati nascosti in byte
    pub compressed: bool, // Indica se il payload è compresso
}

impl Header {
    pub fn new(data_length: usize, compressed: bool) -> Self {
        Header {
            magic: *crate::MAGIC,
            length: data_length as u32,
            compressed,
        }
    }
}
