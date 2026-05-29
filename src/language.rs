use crate::error::{Result, W3StringsError};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct W3Language {
    pub key: u32,
    pub magic: u32,
    pub handle: &'static str,
}

pub static LANGUAGES: &[W3Language] = &[
    W3Language { key: 0x00000000, magic: 0x00000000, handle: "ar" },
    W3Language { key: 0x00000000, magic: 0x00000000, handle: "br" },
    W3Language { key: 0x00000000, magic: 0x00000000, handle: "esMX" },
    W3Language { key: 0x00000000, magic: 0x00000000, handle: "kr" },
    W3Language { key: 0x00000000, magic: 0x00000000, handle: "tr" },
    W3Language { key: 0x83496237, magic: 0x73946816, handle: "pl" },
    W3Language { key: 0x43975139, magic: 0x79321793, handle: "en" },
    W3Language { key: 0x75886138, magic: 0x42791159, handle: "de" },
    W3Language { key: 0x45931894, magic: 0x12375973, handle: "it" },
    W3Language { key: 0x23863176, magic: 0x75921975, handle: "fr" },
    W3Language { key: 0x24987354, magic: 0x21793217, handle: "cz" },
    W3Language { key: 0x18796651, magic: 0x42387566, handle: "es" },
    W3Language { key: 0x18632176, magic: 0x16875467, handle: "zh" },
    W3Language { key: 0x63481486, magic: 0x42386347, handle: "ru" },
    W3Language { key: 0x42378932, magic: 0x67823218, handle: "hu" },
    W3Language { key: 0x54834893, magic: 0x59825646, handle: "jp" },
];

impl W3Language {
    pub fn from_key(key: u32) -> Result<Self> {
        LANGUAGES
            .iter()
            .copied()
            .find(|lang| lang.key == key)
            .ok_or(W3StringsError::UnknownLanguageKey(key))
    }

    pub fn from_handle(handle: &str) -> Result<Self> {
        LANGUAGES
            .iter()
            .copied()
            .find(|lang| lang.handle.eq_ignore_ascii_case(handle))
            .ok_or_else(|| W3StringsError::UnknownLanguageHandle(handle.to_string()))
    }

    pub fn key1(&self) -> u16 {
        (self.key >> 16) as u16
    }

    pub fn key2(&self) -> u16 {
        (self.key & 0xFFFF) as u16
    }
}
