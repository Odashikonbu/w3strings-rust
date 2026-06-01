# w3strings

A fast and safe Rust library for encoding and decoding The Witcher 3 localization files (`.w3strings` / `.wstrings`).

It provides functionality to convert between binary `.w3strings` files and CSV format using pipe-separated values (`|`). It also automatically handles language-specific encryption/decryption keys.

## Features

- **Decode**: Read binary `.w3strings` data into a CSV string. Supports automatic translation key recovery via a hash dictionary.
- **Encode**: Compile a CSV string back into encrypted binary `.w3strings` format.
- **Language Detection**: Automatically matches the language key to the corresponding language settings and Magic numbers.
- **Robust Encryption**: Implements the Dynamic 16-bit shift XOR encryption mechanism used by the Witcher 3 game engine.

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
w3strings = "0.2.0"
```

Or run:
```bash
cargo add w3strings
```

## Usage

### Decoding a `.w3strings` file

```rust
use std::collections::HashMap;
use std::fs;
use w3strings::decode;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let binary_data = fs::read("en.w3strings")?;
    
    // Optional dictionary to map hash values back to key strings (e.g. "panel_Mods")
    let mut hash_dict = HashMap::new();
    hash_dict.insert(w3strings::hash_key("panel_Mods"), "panel_Mods".to_string());
    
    let csv_content = decode(&binary_data, &hash_dict)?;
    
    fs::write("en.w3strings.csv", csv_content)?;
    Ok(())
}
```

### Encoding a CSV back to `.w3strings`

The CSV data must contain the metadata line specifying the language, followed by `|` separated rows:

```csv
;meta[language=en]
; id      |key(hex)|key(str)| text
    174131|77971da9||Stamina damage
2114127085||panel_Mods|Mods
```

```rust
use std::fs;
use w3strings::encode;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let csv_content = fs::read_to_string("en.w3strings.csv")?;
    
    let binary_data = encode(&csv_content)?;
    
    fs::write("en.w3strings", binary_data)?;
    Ok(())
}
```

## CSV Format Specification

The CSV files processed by this crate use `|` (pipe) as the separator:

1. **First Line**: Language metadata, format: `;meta[language=LL]` (where `LL` is a language code like `en`, `jp`, `pl`, `de`, `fr`, etc.)
2. **Second Line**: Headers, format: `; id      |key(hex)|key(str)| text`
3. **Data Lines**: Record rows, format: `<id>|<key_hex>|<key_str>|<text>`
   - `id`: A 32-bit unsigned string ID.
   - `key(hex)`: Optional hex-formatted string key hash.
   - `key(str)`: Optional string key literal (e.g., `panel_Mods`).
   - `text`: The localized string content.

## Credits

- Based on the original concepts and logic by the authors of `w3strings encoder`.
- Implementation references the C# codebase of the [WolvenKit](https://github.com/WolvenKit/WolvenKit-7) project.

## License

This project is licensed under the GPL-3.0 License - see the [LICENSE](LICENSE) file for details.
