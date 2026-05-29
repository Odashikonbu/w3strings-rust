pub fn hash_key(key: &str) -> u32 {
    let lower = key.to_lowercase();
    let mut hash = 0u32;
    for c in lower.encode_utf16() {
        hash = hash.wrapping_mul(31).wrapping_add(c as u32);
    }
    hash
}
