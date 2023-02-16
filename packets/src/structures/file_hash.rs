use serde::{Deserialize, Serialize};
use std::fmt::{Debug, Display};

#[derive(Default, Hash, PartialEq, Eq, Clone, Copy, Serialize, Deserialize)]
pub struct FileHash {
    bytes: [u8; 16],
}

impl FileHash {
    pub const ZERO: FileHash = FileHash { bytes: [0; 16] };

    pub fn hash(data: &[u8]) -> Self {
        Self {
            bytes: md5::compute(data).0,
        }
    }

    pub fn from_hex(value: &str) -> Option<Self> {
        let bytes = hex::decode(value).ok()?;
        let bytes = bytes.try_into().ok()?;

        Some(Self { bytes })
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes
    }
}

impl Display for FileHash {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for byte in self.bytes {
            write!(f, "{byte:02x}")?;
        }

        Ok(())
    }
}

impl Debug for FileHash {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(self, f)
    }
}
