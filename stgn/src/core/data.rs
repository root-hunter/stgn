use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
pub enum DataType {
    Text,
    Binary,
}

#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
pub struct DataElement {
    pub name: String,
    pub data_type: DataType,
    pub value: Vec<u8>,
}

impl DataElement {
    pub fn text(name: impl Into<String>, content: &str) -> Self {
        DataElement {
            name: name.into(),
            data_type: DataType::Text,
            value: content.as_bytes().to_vec(),
        }
    }

    pub fn bytes(name: impl Into<String>, content: Vec<u8>) -> Self {
        DataElement {
            name: name.into(),
            data_type: DataType::Binary,
            value: content,
        }
    }

    /// Returns the raw bytes of this entry.
    pub fn as_bytes(&self) -> &[u8] {
        &self.value
    }

    /// Tries to interpret the value as a UTF-8 string.
    pub fn as_str(&self) -> Result<&str, std::str::Utf8Error> {
        std::str::from_utf8(&self.value)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
pub struct Data {
    pub elements: Vec<DataElement>,
}

impl Data {
    pub fn new() -> Self {
        Data {
            elements: Vec::new(),
        }
    }

    /// Builder-style push.
    pub fn add(mut self, entry: DataElement) -> Self {
        self.elements.push(entry);
        self
    }

    pub fn push(&mut self, entry: DataElement) {
        self.elements.push(entry);
    }

    /// Shortcut: single text entry named "message".
    pub fn from_text(content: &str) -> Self {
        Data::new().add(DataElement::text("message", content))
    }

    /// Shortcut: single binary entry named "data".
    pub fn from_bytes_payload(content: Vec<u8>) -> Self {
        Data::new().add(DataElement::bytes("data", content))
    }

    /// Shortcut: single binary entry with the given filename.
    pub fn from_file(name: impl Into<String>, content: Vec<u8>) -> Self {
        Data::new().add(DataElement::bytes(name, content))
    }

    // ── Lookup helpers ────────────────────────────────────────────────────────

    pub fn get(&self, name: &str) -> Option<&DataElement> {
        self.elements.iter().find(|e| e.name == name)
    }

    pub fn get_text(&self, name: &str) -> Option<&str> {
        self.get(name).and_then(|e| e.as_str().ok())
    }

    pub fn get_bytes(&self, name: &str) -> Option<&[u8]> {
        self.get(name).map(|e| e.as_bytes())
    }

    /// Returns the first entry's bytes (for single-payload convenience).
    pub fn first_bytes(&self) -> Option<&[u8]> {
        self.elements.first().map(|e| e.as_bytes())
    }

    /// Returns the first text-typed entry as &str.
    pub fn first_text(&self) -> Option<&str> {
        self.elements.iter().find_map(|e| {
            if matches!(e.data_type, DataType::Text) {
                e.as_str().ok()
            } else {
                None
            }
        })
    }

    /// Returns the first entry as a String (tries UTF-8 on any type).
    pub fn first_as_string(&self) -> Option<String> {
        self.elements
            .first()
            .and_then(|e| e.as_str().ok())
            .map(|s| s.to_string())
    }
}

impl Default for Data {
    fn default() -> Self {
        Data::new()
    }
}
