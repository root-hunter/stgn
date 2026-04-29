use aes_gcm::aead::rand_core::RngCore;
use aes_gcm::aead::{Aead, KeyInit, OsRng};
use aes_gcm::{Aes256Gcm, Key, Nonce};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
pub enum EncryptionType {
    None,
    Aes256,
}

pub enum EncryptionSecret {
    None,
    Aes256(Vec<u8>), // deve essere esattamente 32 byte
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
pub struct SecureContext {
    pub encryption_type: EncryptionType,
}

impl SecureContext {
    pub fn new(encryption_type: EncryptionType) -> Self {
        SecureContext { encryption_type }
    }

    pub fn encrypt(
        &self,
        data: &[u8],
        secret: &EncryptionSecret,
    ) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        match (&self.encryption_type, secret) {
            (EncryptionType::None, _) => Ok(data.to_vec()),
            (EncryptionType::Aes256, EncryptionSecret::Aes256(key)) => {
                let key = Key::<Aes256Gcm>::from_slice(&key[..32]);
                let cipher = Aes256Gcm::new(key);
                let mut nonce_bytes = [0u8; 12];
                OsRng.fill_bytes(&mut nonce_bytes);
                let nonce = Nonce::from_slice(&nonce_bytes);
                let ciphertext = cipher
                    .encrypt(nonce, data)
                    .map_err(|e| format!("AES-256-GCM encrypt error: {e}"))?;
                let mut out = Vec::with_capacity(12 + ciphertext.len());
                out.extend_from_slice(&nonce_bytes);
                out.extend_from_slice(&ciphertext);
                Ok(out)
            }
            _ => Err("Mismatched encryption type and secret".into()),
        }
    }

    pub fn decrypt(
        &self,
        data: &[u8],
        secret: &EncryptionSecret,
    ) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        match (&self.encryption_type, secret) {
            (EncryptionType::None, _) => Ok(data.to_vec()),
            (EncryptionType::Aes256, EncryptionSecret::Aes256(key)) => {
                if data.len() < 12 {
                    return Err("Ciphertext too short (missing nonce)".into());
                }
                let (nonce_bytes, ciphertext) = data.split_at(12);
                let key = Key::<Aes256Gcm>::from_slice(&key[..32]);
                let cipher = Aes256Gcm::new(key);
                let nonce = Nonce::from_slice(nonce_bytes);
                cipher
                    .decrypt(nonce, ciphertext)
                    .map_err(|e| format!("AES-256-GCM decrypt error: {e}").into())
            }
            _ => Err("Mismatched encryption type and secret".into()),
        }
    }
}
