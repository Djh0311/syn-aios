use sha2::{Digest, Sha256};

pub(crate) fn sha256_hex(value: &str) -> String {
    sha256_hex_bytes(value.as_bytes())
}

pub(crate) fn sha256_hex_bytes(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("{:x}", hasher.finalize())
}

pub(crate) fn short_hash(value: &str) -> String {
    short_hash_len(value, 16)
}

pub(crate) fn short_hash12(value: &str) -> String {
    short_hash_len(value, 12)
}

pub(crate) fn short_hash_len(value: &str, len: usize) -> String {
    sha256_hex(value).chars().take(len).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hash_helpers_preserve_existing_lengths() {
        let value = "workbench-hash-helper";
        let full = sha256_hex(value);

        assert_eq!(full.len(), 64);
        assert_eq!(sha256_hex_bytes(value.as_bytes()), full);
        assert_eq!(short_hash(value), full.chars().take(16).collect::<String>());
        assert_eq!(
            short_hash12(value),
            full.chars().take(12).collect::<String>()
        );
    }
}
