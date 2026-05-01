//! JPEG steganography via J-UNIWARD + STC (juniward crate).
//!
//! Unlike the LSB encoder that works on raw pixel buffers (`DynamicImage`),
//! this module operates directly on **JPEG byte streams** so that the DCT
//! domain is preserved end-to-end.
//!
//! # Example
//! ```no_run
//! use stgn::embedding::jpeg::JpegEmbedding;
//!
//! let cover_jpeg = std::fs::read("cover.jpg").unwrap();
//! let secret = b"my secret message";
//!
//! // Embed — returns (stego_jpeg, frame_len)
//! let (stego_jpeg, frame_len) = JpegEmbedding::embed(&cover_jpeg, secret, None).unwrap();
//! std::fs::write("stego.jpg", &stego_jpeg).unwrap();
//!
//! // Extract
//! let recovered = JpegEmbedding::extract(&stego_jpeg, frame_len, None).unwrap();
//! assert_eq!(recovered, secret);
//! ```

use flate2::{Compression, read::DeflateDecoder, write::DeflateEncoder};
use juniward::{EmbedConfig, StcParams, embed, embed_with_params, extract_with_params};
use postcard::{from_bytes, to_allocvec};
use std::io::{Read, Write};
use std::os::raw::c_ulong;


use crate::core::{
    auth::{EncryptionSecret, EncryptionType, SecureContext},
    data::Data,
    header::Header,
};

/// Errors produced by [`JpegEmbedding`].
#[derive(Debug)]
pub enum JpegEmbeddingError {
    /// The raw bytes are not a valid JPEG.
    InvalidJpeg(String),
    /// The serialised payload does not fit in the cover JPEG at the chosen embedding rate.
    PayloadTooLarge {
        payload_bits: usize,
        max_bits: usize,
    },
    /// J-UNIWARD / STC internal error.
    EmbeddingFailed(String),
    /// Any other I/O or serialisation error.
    Other(String),
}

impl std::fmt::Display for JpegEmbeddingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidJpeg(s) => write!(f, "Invalid JPEG: {s}"),
            Self::PayloadTooLarge {
                payload_bits,
                max_bits,
            } => write!(
                f,
                "Payload too large: {payload_bits} bits > max {max_bits} bits"
            ),
            Self::EmbeddingFailed(s) => write!(f, "Embedding failed: {s}"),
            Self::Other(s) => write!(f, "{s}"),
        }
    }
}

impl std::error::Error for JpegEmbeddingError {}

impl From<juniward::JuniwardError> for JpegEmbeddingError {
    fn from(e: juniward::JuniwardError) -> Self {
        match e {
            juniward::JuniwardError::InvalidJpeg(s) => Self::InvalidJpeg(s),
            juniward::JuniwardError::PayloadTooLarge {
                payload_bits,
                max_bits,
            } => Self::PayloadTooLarge {
                payload_bits,
                max_bits,
            },
            juniward::JuniwardError::EmbeddingFailed(e) => Self::EmbeddingFailed(e.to_string()),
        }
    }
}

impl From<Box<dyn std::error::Error>> for JpegEmbeddingError {
    fn from(e: Box<dyn std::error::Error>) -> Self {
        Self::Other(e.to_string())
    }
}

// ─── Configuration ────────────────────────────────────────────────────────────

/// Configuration forwarded to the J-UNIWARD pipeline.
pub struct JpegEmbedConfig {
    /// J-UNIWARD sigma regularisation constant (default: 1e-10).
    pub sigma: f64,
    /// STC trellis height (default: 7).
    pub stc_h_height: usize,
    /// Maximum bits-per-non-zero-AC-coefficient safety cap (default: 0.4).
    pub max_bpnzac: f64,
    /// Whether to compress the payload with Deflate before embedding (default: true).
    pub compress: bool,
    /// Optional password used to derive the STC parity-check matrix H.
    ///
    /// When `Some`, the H matrix and the coefficient permutation are derived
    /// from this password via SHA-256.  The **same password** must be supplied
    /// during extraction, otherwise the output is garbage.
    ///
    /// When `None`, the well-known literature values are used (⚠️ insecure).
    pub password: Option<String>,
    /// Number of distinct H columns derived from the password (default: 8).
    /// Ignored when `password` is `None`.
    pub n_stc_cols: usize,
}

impl Default for JpegEmbedConfig {
    fn default() -> Self {
        Self {
            sigma: 1e-10,
            stc_h_height: 7,
            max_bpnzac: 0.4,
            compress: true,
            password: None,
            n_stc_cols: 8,
        }
    }
}

impl JpegEmbedConfig {
    /// Convenience constructor: defaults + password.
    pub fn with_password(password: impl Into<String>) -> Self {
        Self {
            password: Some(password.into()),
            ..Self::default()
        }
    }
}

// ─── Public API ───────────────────────────────────────────────────────────────

/// JPEG steganography embedding / extraction via J-UNIWARD + STC.
pub struct JpegEmbedding;

impl JpegEmbedding {
    // ── Internal helpers ─────────────────────────────────────────────────────

    /// Serialises a [`Data`] payload into raw bytes, optionally compresses and
    /// encrypts it, then prepends the auth + header framing used by the LSB codec.
    fn build_frame(
        data: &Data,
        secret: Option<&EncryptionSecret>,
        compress: bool,
    ) -> Result<Vec<u8>, JpegEmbeddingError> {
        let serialized = to_allocvec(data).map_err(|e| JpegEmbeddingError::Other(e.to_string()))?;

        let auth = SecureContext::new(match secret {
            Some(EncryptionSecret::Aes256(_)) => EncryptionType::Aes256,
            _ => EncryptionType::None,
        });

        let compressed = if compress {
            let mut enc = DeflateEncoder::new(Vec::new(), Compression::default());
            enc.write_all(&serialized)
                .map_err(|e| JpegEmbeddingError::Other(e.to_string()))?;
            enc.finish()
                .map_err(|e| JpegEmbeddingError::Other(e.to_string()))?
        } else {
            serialized
        };

        let payload = if secret.is_some() && !matches!(auth.encryption_type, EncryptionType::None) {
            auth.encrypt(&compressed, secret.unwrap())
                .map_err(|e| JpegEmbeddingError::Other(e.to_string()))?
        } else {
            compressed
        };

        let mut auth_buf = [0u8; 16];
        let auth_bytes = postcard::to_slice(&auth, &mut auth_buf)
            .map_err(|e| JpegEmbeddingError::Other(e.to_string()))?;
        let auth_len = auth_bytes.len() as u8;

        let header = Header::new(payload.len(), compress);
        let mut header_buf = [0u8; 16];
        let header_bytes = postcard::to_slice(&header, &mut header_buf)
            .map_err(|e| JpegEmbeddingError::Other(e.to_string()))?;
        let header_len = header_bytes.len() as u8;

        let mut frame: Vec<u8> = Vec::new();
        frame.push(auth_len);
        frame.extend_from_slice(auth_bytes);
        frame.push(header_len);
        frame.extend_from_slice(header_bytes);
        frame.extend_from_slice(&payload);

        Ok(frame)
    }

    /// Parses the framed bytes back into a [`Data`] payload.
    fn parse_frame(
        frame: &[u8],
        secret: Option<&EncryptionSecret>,
    ) -> Result<Data, JpegEmbeddingError> {
        let mut pos = 0;

        let auth_len = frame[pos] as usize;
        pos += 1;
        let auth: SecureContext = from_bytes(&frame[pos..pos + auth_len])
            .map_err(|e| JpegEmbeddingError::Other(e.to_string()))?;
        pos += auth_len;

        let header_len = frame[pos] as usize;
        pos += 1;
        let header: Header = from_bytes(&frame[pos..pos + header_len])
            .map_err(|e| JpegEmbeddingError::Other(e.to_string()))?;
        assert_eq!(
            header.magic,
            *crate::MAGIC,
            "Invalid magic bytes in JPEG stego frame"
        );
        pos += header_len;

        let payload = &frame[pos..pos + header.length as usize];

        let decrypted = if !matches!(auth.encryption_type, EncryptionType::None) {
            let s = secret.ok_or_else(|| {
                JpegEmbeddingError::Other("Secret required for decryption".into())
            })?;
            auth.decrypt(payload, s)
                .map_err(|e| JpegEmbeddingError::Other(e.to_string()))?
        } else {
            payload.to_vec()
        };

        let raw = if header.compressed {
            let mut dec = DeflateDecoder::new(&decrypted[..]);
            let mut out = Vec::new();
            dec.read_to_end(&mut out)
                .map_err(|e| JpegEmbeddingError::Other(e.to_string()))?;
            out
        } else {
            decrypted
        };

        from_bytes(&raw).map_err(|e| JpegEmbeddingError::Other(e.to_string()))
    }

    // ── Core public methods ───────────────────────────────────────────────────

    /// Embeds a [`Data`] payload into a JPEG byte stream using J-UNIWARD + STC.
    ///
    /// Returns the stego JPEG as a byte vector.
    pub fn embed_payload(
        cover_jpeg: &[u8],
        data: &Data,
        secret: Option<&EncryptionSecret>,
        cfg: Option<JpegEmbedConfig>,
    ) -> Result<Vec<u8>, JpegEmbeddingError> {
        let cfg = cfg.unwrap_or_default();
        let frame = Self::build_frame(data, secret, cfg.compress)?;

        let juniward_cfg = EmbedConfig {
            sigma: cfg.sigma,
            stc_h_height: cfg.stc_h_height,
            max_bpnzac: cfg.max_bpnzac,
        };

        match cfg.password {
            Some(pw) => {
                let params = StcParams::from_key(pw.as_bytes(), cfg.stc_h_height, cfg.n_stc_cols);
                embed_with_params(cover_jpeg, &frame, juniward_cfg, &params).map_err(Into::into)
            }
            None => embed(cover_jpeg, &frame, juniward_cfg).map_err(Into::into),
        }
    }

    /// Extracts a [`Data`] payload from a stego JPEG.
    ///
    /// `frame_len` must equal the **byte length** of the framed message
    /// (i.e. the value returned by [`Self::frame_len`]).
    /// Both sender and receiver must agree on this value.
    ///
    /// `password` — if the image was embedded with a password, supply the same
    /// value here; otherwise extraction will produce garbage.
    pub fn extract_payload(
        stego_jpeg: &[u8],
        frame_len: usize,
        secret: Option<&EncryptionSecret>,
        password: Option<&str>,
    ) -> Result<Data, JpegEmbeddingError> {
        let params = match password {
            Some(pw) => StcParams::from_key(pw.as_bytes(), 7, 8),
            None => StcParams::new(7),
        };
        let frame = extract_with_params(stego_jpeg, frame_len, &params)?;
        Self::parse_frame(&frame, secret)
    }

    // ── Convenience wrappers ──────────────────────────────────────────────────

    /// Embeds raw bytes into a JPEG cover image.
    ///
    /// Returns `(stego_jpeg, frame_len)` — keep `frame_len` to pass to [`Self::extract`].
    pub fn embed(
        cover_jpeg: &[u8],
        message: &[u8],
        secret: Option<&EncryptionSecret>,
    ) -> Result<(Vec<u8>, usize), JpegEmbeddingError> {
        Self::embed_with_cfg(cover_jpeg, message, secret, None)
    }

    /// Like [`embed`] but accepts a full [`JpegEmbedConfig`] (including password).
    pub fn embed_with_cfg(
        cover_jpeg: &[u8],
        message: &[u8],
        secret: Option<&EncryptionSecret>,
        cfg: Option<JpegEmbedConfig>,
    ) -> Result<(Vec<u8>, usize), JpegEmbeddingError> {
        let data = Data::from_bytes_payload(message.to_vec());
        let cfg = cfg.unwrap_or_default();
        let frame = Self::build_frame(&data, secret, cfg.compress)?;
        let frame_len = frame.len();
        let juniward_cfg = EmbedConfig {
            sigma: cfg.sigma,
            stc_h_height: cfg.stc_h_height,
            max_bpnzac: cfg.max_bpnzac,
        };
        let stego = match cfg.password {
            Some(pw) => {
                let params = StcParams::from_key(pw.as_bytes(), cfg.stc_h_height, cfg.n_stc_cols);
                embed_with_params(cover_jpeg, &frame, juniward_cfg, &params)?
            }
            None => embed(cover_jpeg, &frame, juniward_cfg)?,
        };
        Ok((stego, frame_len))
    }

    /// Extracts raw bytes from a stego JPEG.
    ///
    /// `frame_len` is the second element of the tuple returned by [`Self::embed`].
    /// Supply the same `password` used during embedding (or `None` if none was used).
    pub fn extract(
        stego_jpeg: &[u8],
        frame_len: usize,
        secret: Option<&EncryptionSecret>,
    ) -> Result<Vec<u8>, JpegEmbeddingError> {
        Self::extract_with_password(stego_jpeg, frame_len, secret, None)
    }

    /// Like [`extract`] but accepts a password to reconstruct the keyed H matrix.
    pub fn extract_with_password(
        stego_jpeg: &[u8],
        frame_len: usize,
        secret: Option<&EncryptionSecret>,
        password: Option<&str>,
    ) -> Result<Vec<u8>, JpegEmbeddingError> {
        let data = Self::extract_payload(stego_jpeg, frame_len, secret, password)?;
        data.get_bytes("data")
            .map(|b| b.to_vec())
            .ok_or_else(|| JpegEmbeddingError::Other("No 'data' entry found in payload".into()))
    }

    /// Embeds a UTF-8 string into a JPEG cover image.
    ///
    /// Returns `(stego_jpeg, frame_len)`.
    pub fn embed_string(
        cover_jpeg: &[u8],
        text: &str,
        secret: Option<&EncryptionSecret>,
    ) -> Result<(Vec<u8>, usize), JpegEmbeddingError> {
        Self::embed_string_with_cfg(cover_jpeg, text, secret, None)
    }

    /// Like [`embed_string`] but accepts a full [`JpegEmbedConfig`] (including password).
    pub fn embed_string_with_cfg(
        cover_jpeg: &[u8],
        text: &str,
        secret: Option<&EncryptionSecret>,
        cfg: Option<JpegEmbedConfig>,
    ) -> Result<(Vec<u8>, usize), JpegEmbeddingError> {
        let data = Data::from_text(text);
        let cfg = cfg.unwrap_or_default();
        let frame = Self::build_frame(&data, secret, cfg.compress)?;
        let frame_len = frame.len();
        let juniward_cfg = EmbedConfig {
            sigma: cfg.sigma,
            stc_h_height: cfg.stc_h_height,
            max_bpnzac: cfg.max_bpnzac,
        };
        let stego = match cfg.password {
            Some(pw) => {
                let params = StcParams::from_key(pw.as_bytes(), cfg.stc_h_height, cfg.n_stc_cols);
                embed_with_params(cover_jpeg, &frame, juniward_cfg, &params)?
            }
            None => embed(cover_jpeg, &frame, juniward_cfg)?,
        };
        Ok((stego, frame_len))
    }

    /// Extracts a UTF-8 string from a stego JPEG.
    /// Supply the same `password` used during embedding (or `None` if none was used).
    pub fn extract_string(
        stego_jpeg: &[u8],
        frame_len: usize,
        secret: Option<&EncryptionSecret>,
    ) -> Result<String, JpegEmbeddingError> {
        Self::extract_string_with_password(stego_jpeg, frame_len, secret, None)
    }

    /// Like [`extract_string`] but accepts a password to reconstruct the keyed H matrix.
    pub fn extract_string_with_password(
        stego_jpeg: &[u8],
        frame_len: usize,
        secret: Option<&EncryptionSecret>,
        password: Option<&str>,
    ) -> Result<String, JpegEmbeddingError> {
        let data = Self::extract_payload(stego_jpeg, frame_len, secret, password)?;
        data.get_text("message")
            .map(|s| s.to_string())
            .ok_or_else(|| JpegEmbeddingError::Other("No 'message' entry found in payload".into()))
    }

    /// Computes the frame byte length that the receiver needs to pass to extract.
    ///
    /// Useful when you want to build the frame size out-of-band before embedding.
    pub fn frame_len(
        data: &Data,
        secret: Option<&EncryptionSecret>,
        compress: bool,
    ) -> Result<usize, JpegEmbeddingError> {
        Ok(Self::build_frame(data, secret, compress)?.len())
    }
}
