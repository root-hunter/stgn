use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
pub enum EncryptionType {
    None,
    Xor,
    Aes256,
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
pub struct SecureContext {
    pub encryption_type: EncryptionType,
}

impl SecureContext {
    pub fn new(encryption_type: EncryptionType) -> Self {
        SecureContext { encryption_type }
    }

    pub fn encrypt(&self, data: &[u8], secret: &EncryptionSecret) -> Vec<u8> {
        match (&self.encryption_type, secret) {
            (EncryptionType::None, _) => data.to_vec(),
            (EncryptionType::Xor, EncryptionSecret::Xor(key)) => {
                data.iter().zip(key.iter().cycle()).map(|(b, k)| b ^ k).collect()
            }
            (EncryptionType::Aes256, EncryptionSecret::Aes256(key)) => {
                // Placeholder: implement AES-256 encryption
                unimplemented!()
            }
            _ => panic!("Mismatched encryption type and secret"),
        }
    }

    pub fn decrypt(&self, data: &[u8], secret: &EncryptionSecret) -> Vec<u8> {
        match (&self.encryption_type, secret) {
            (EncryptionType::None, _) => data.to_vec(),
            (EncryptionType::Xor, EncryptionSecret::Xor(key)) => {
                data.iter().zip(key.iter().cycle()).map(|(b, k)| b ^ k).collect()
            }
            (EncryptionType::Aes256, EncryptionSecret::Aes256(key)) => {
                // Placeholder: implement AES-256 decryption
                unimplemented!()
            }
            _ => panic!("Mismatched encryption type and secret"),
        }
    }
}

pub enum EncryptionSecret {
    Xor(Vec<u8>),
    Aes256(Vec<u8>),
}