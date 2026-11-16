//! Cryptographic hashing with SHA-256 and domain separation.

use sha2::{Digest, Sha256};

pub const MERKLE_LEAF_PREFIX: u8 = 0x00;
pub const MERKLE_NODE_PREFIX: u8 = 0x01;
pub const MANIFEST_HASH_PREFIX: &[u8] = b"QSHIELD-MANIFEST-SHA256:";
pub const TILE_HASH_PREFIX: &[u8] = b"QSHIELD-TILE-SHA256:";

/// Compute standard SHA-256 digest of data
pub fn sha256_digest(data: &[u8]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hasher.finalize().into()
}

/// Compute SHA-256 digest formatted as hex string
pub fn sha256_hex(data: &[u8]) -> String {
    hex::encode(sha256_digest(data))
}

/// Domain-separated SHA-256 for Merkle leaves: SHA-256(0x00 || leaf_data)
pub fn hash_merkle_leaf(leaf_data: &[u8]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update([MERKLE_LEAF_PREFIX]);
    hasher.update(leaf_data);
    hasher.finalize().into()
}

/// Domain-separated SHA-256 for Merkle internal nodes: SHA-256(0x01 || left || right)
pub fn hash_merkle_node(left: &[u8; 32], right: &[u8; 32]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update([MERKLE_NODE_PREFIX]);
    hasher.update(left);
    hasher.update(right);
    hasher.finalize().into()
}

/// Compute domain-separated tile hash
pub fn hash_tile(tile_bytes: &[u8]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(TILE_HASH_PREFIX);
    hasher.update(tile_bytes);
    hasher.finalize().into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_domain_separation() {
        let data = b"sample_tile_payload";
        let leaf_hash = hash_merkle_leaf(data);
        let direct_hash = sha256_digest(data);

        // Domain prefix ensures leaf_hash != direct_hash
        assert_ne!(leaf_hash, direct_hash);

        // Two identical nodes give deterministic parent
        let node1 = hash_merkle_node(&leaf_hash, &leaf_hash);
        let node2 = hash_merkle_node(&leaf_hash, &leaf_hash);
        assert_eq!(node1, node2);
    }
}
