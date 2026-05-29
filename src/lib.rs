pub mod error;
pub mod language;
pub mod vlq;
pub mod hash;
pub mod file;

pub use crate::error::{Result, W3StringsError};
pub use crate::language::{W3Language, LANGUAGES};
pub use crate::hash::hash_key;
pub use crate::file::{W3StringFile, W3StringBlock1, W3StringBlock2};

use std::collections::HashMap;
use std::io::Cursor;

/// Decodes binary `w3strings` data into a CSV string (separator: `|`).
///
/// An optional `hash_dictionary` can be provided to map numeric hash keys
/// back to their original string identifiers (e.g. `panel_Mods`).
pub fn decode(data: &[u8], hash_dictionary: &HashMap<u32, String>) -> Result<String> {
    let mut cursor = Cursor::new(data);
    let file = W3StringFile::read(&mut cursor)?;
    
    let mut csv_out = String::new();
    csv_out.push_str(&format!(";meta[language={}]\n", file.language.handle));
    csv_out.push_str("; id      |key(hex)|key(str)| text\n");
    
    let mut block2_map = HashMap::new();
    for b2 in &file.block2 {
        block2_map.insert(b2.str_id, b2.str_key_hex);
    }
    
    let mut sorted_block1 = file.block1.clone();
    sorted_block1.sort_by_key(|b| b.offset);
    
    for b1 in &sorted_block1 {
        let (key_hex, key_str) = if let Some(&str_key_hex) = block2_map.get(&b1.str_id) {
            if let Some(s) = hash_dictionary.get(&str_key_hex) {
                ("".to_string(), s.clone())
            } else {
                (format!("{:08x}", str_key_hex), "".to_string())
            }
        } else {
            ("00000000".to_string(), "".to_string())
        };
        
        csv_out.push_str(&format!(
            "{:>10}|{}|{}|{}\n",
            b1.str_id, key_hex, key_str, b1.str
        ));
    }
    
    Ok(csv_out)
}

/// Encodes CSV string data (separator: `|`) into binary `w3strings` format.
pub fn encode(csv_data: &str) -> Result<Vec<u8>> {
    let mut lines = csv_data.lines();
    
    let meta_line = lines.next().ok_or(W3StringsError::InvalidVlq)?; // Empty or missing meta
    if !meta_line.starts_with(";meta[language=") || !meta_line.ends_with(']') {
        return Err(W3StringsError::UnknownLanguageHandle("missing or invalid meta line".to_string()));
    }
    let lang_handle = &meta_line[";meta[language=".len()..meta_line.len() - 1];
    let language = W3Language::from_handle(lang_handle)?;
    
    let _header_line = lines.next().ok_or_else(|| {
        W3StringsError::UnknownLanguageHandle("missing header line".to_string())
    })?;
    
    let mut block1 = Vec::new();
    let mut block2 = Vec::new();
    
    for line in lines {
        if line.trim().is_empty() {
            continue;
        }
        let parts: Vec<&str> = line.splitn(4, '|').collect();
        if parts.len() < 4 {
            continue; // Skip invalid or empty lines
        }
        
        let id_str = parts[0].trim();
        let key_hex_str = parts[1].trim();
        let key_str = parts[2].trim();
        let text = parts[3];
        
        let id = id_str.parse::<u32>().map_err(|_| {
            std::io::Error::new(std::io::ErrorKind::InvalidData, format!("invalid id: {}", id_str))
        })?;
        
        let mut str_key_hex = None;
        if !key_str.is_empty() {
            let hash = hash_key(key_str);
            if hash != 0 {
                str_key_hex = Some(hash);
            }
        } else if !key_hex_str.is_empty() {
            let hex = u32::from_str_radix(key_hex_str, 16).map_err(|_| {
                std::io::Error::new(std::io::ErrorKind::InvalidData, format!("invalid hex key: {}", key_hex_str))
            })?;
            if hex != 0 {
                str_key_hex = Some(hex);
            }
        }
        
        block1.push(W3StringBlock1 {
            str_id: id,
            offset: 0,
            strlen: 0,
            str: text.to_string(),
        });
        
        if let Some(hex) = str_key_hex {
            block2.push(W3StringBlock2 {
                str_key_hex: hex,
                str_id: id,
            });
        }
    }
    
    let key1 = language.key1();
    let key2 = language.key2();
    
    let file = W3StringFile {
        version: 162,
        key1,
        key2,
        language,
        block1,
        block2,
        block3count: 0,
    };
    
    let mut out = Cursor::new(Vec::new());
    file.write(&mut out)?;
    Ok(out.into_inner())
}
