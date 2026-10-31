//! SHA-256 helpers.

use sha2::{Digest, Sha256};

pub fn sha256_hex(bytes: &[u8]) -> String {
    let mut h = Sha256::new();
    h.update(bytes);
    let dig = h.finalize();
    dig.iter().map(|b| format!("{:02x}", b)).collect()
}

pub fn sha256_str(s: &str) -> String {
    sha256_hex(s.as_bytes())
}
