use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use w3strings::{decode, encode, hash_key};

fn print_usage() {
    let program_name = std::env::args().next()
        .and_then(|p| std::path::Path::new(&p).file_name().map(|n| n.to_string_lossy().into_owned()))
        .unwrap_or_else(|| "w3strings-ng".to_string());
    eprintln!("Usage:");
    eprintln!("  {} encode <CSV_path> <w3strings_path>", program_name);
    eprintln!("  {} decode <w3strings_path> <CSV_path>", program_name);
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 4 {
        print_usage();
        std::process::exit(1);
    }

    let command = &args[1];
    match command.as_str() {
        "encode" => {
            let csv_path = &args[2];
            let w3strings_path = &args[3];
            
            println!("Encoding {} to {}...", csv_path, w3strings_path);
            let csv_content = fs::read_to_string(csv_path)?;
            let encoded_data = encode(&csv_content)?;
            fs::write(w3strings_path, encoded_data)?;
            println!("Encoding completed successfully.");
        }
        "decode" => {
            let w3strings_path = &args[2];
            let csv_path = &args[3];
            
            println!("Decoding {} to {}...", w3strings_path, csv_path);
            let w3strings_data = fs::read(w3strings_path)?;
            
            // Try to load a dictionary to resolve hash keys
            let hash_dict = load_dictionary(w3strings_path);
            if !hash_dict.is_empty() {
                println!("Loaded {} keys from dictionary.", hash_dict.len());
            }
            
            let decoded_csv = decode(&w3strings_data, &hash_dict)?;
            fs::write(csv_path, decoded_csv)?;
            println!("Decoding completed successfully.");
        }
        _ => {
            eprintln!("Unknown command: {}", command);
            print_usage();
            std::process::exit(1);
        }
    }

    Ok(())
}

fn load_dictionary(w3strings_path: &str) -> HashMap<u32, String> {
    let mut hash_dict = HashMap::new();
    
    // We search for a dictionary file named "w3strings.txt" or "vanilla.w3strings.txt" in:
    // 1. The same directory as the target w3strings file.
    // 2. The current working directory.
    // 3. The directory where the executable is located.
    let mut search_paths = Vec::new();
    
    // 1. Target directory
    if let Some(parent) = Path::new(w3strings_path).parent() {
        search_paths.push(parent.join("w3strings.txt"));
        search_paths.push(parent.join("vanilla.w3strings.txt"));
    }
    
    // 2. Current working directory
    search_paths.push(PathBuf::from("w3strings.txt"));
    search_paths.push(PathBuf::from("vanilla.w3strings.txt"));
    
    // 3. Executable directory
    if let Ok(exe_path) = env::current_exe() {
        if let Some(exe_dir) = exe_path.parent() {
            search_paths.push(exe_dir.join("w3strings.txt"));
            search_paths.push(exe_dir.join("vanilla.w3strings.txt"));
        }
    }
    
    for path in search_paths {
        if path.exists() {
            if let Ok(content) = fs::read_to_string(&path) {
                println!("Found dictionary at: {:?}", path);
                for line in content.lines() {
                    let trimmed = line.trim();
                    if trimmed.is_empty() || trimmed.starts_with(';') || trimmed.starts_with('#') {
                        continue;
                    }
                    
                    // Support both plain string keys list AND CSV dump files
                    if trimmed.contains('|') {
                        let parts: Vec<&str> = trimmed.split('|').collect();
                        if parts.len() >= 3 {
                            let key_str = parts[2].trim();
                            if !key_str.is_empty() {
                                let hash = hash_key(key_str);
                                hash_dict.insert(hash, key_str.to_string());
                            }
                        }
                    } else {
                        let hash = hash_key(trimmed);
                        hash_dict.insert(hash, trimmed.to_string());
                    }
                }
                // Break after loading the first found dictionary
                break;
            }
        }
    }
    
    hash_dict
}
