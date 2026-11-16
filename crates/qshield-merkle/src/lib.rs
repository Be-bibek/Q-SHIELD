//! Q-SHIELD Domain-Separated Merkle Tree and Tile Inclusion Proof Engine.

use qshield_core::{ImageId, QShieldError, Result, TileCoordinate, TileId, TransferId};
use qshield_crypto::{hash_merkle_leaf, hash_merkle_node};
use serde::{Deserialize, Serialize};

/// Canonical leaf representation binding tile identity, coordinates, dimensions, and transfer ID
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MerkleLeafBinding {
    pub transfer_id: TransferId,
    pub image_id: ImageId,
    pub tile_id: TileId,
    pub row: u32,
    pub col: u32,
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
    pub encoding: String,
    pub tile_hash: [u8; 32],
}

impl MerkleLeafBinding {
    pub fn new(
        transfer_id: TransferId,
        image_id: &ImageId,
        coord: &TileCoordinate,
        tile_hash: [u8; 32],
        encoding: impl Into<String>,
    ) -> Self {
        Self {
            transfer_id,
            image_id: image_id.clone(),
            tile_id: coord.tile_id,
            row: coord.row,
            col: coord.col,
            x: coord.x,
            y: coord.y,
            width: coord.width,
            height: coord.height,
            encoding: encoding.into(),
            tile_hash,
        }
    }

    /// Construct unambiguous canonical representation for hashing
    pub fn canonical_bytes(&self) -> Vec<u8> {
        format!(
            "LEAF:TID={}:IMG={}:TID_NUM={}:R={}:C={}:X={}:Y={}:W={}:H={}:ENC={}:TH={}",
            self.transfer_id,
            self.image_id,
            self.tile_id.0,
            self.row,
            self.col,
            self.x,
            self.y,
            self.width,
            self.height,
            self.encoding,
            hex::encode(self.tile_hash)
        )
        .into_bytes()
    }

    /// Compute domain-separated leaf hash
    pub fn compute_leaf_hash(&self) -> [u8; 32] {
        hash_merkle_leaf(&self.canonical_bytes())
    }
}

/// Sibling node in a Merkle inclusion proof
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MerkleProofStep {
    pub sibling_hash: [u8; 32],
    pub is_left: bool,
}

/// Merkle inclusion proof for an individual image tile
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MerkleInclusionProof {
    pub leaf_index: usize,
    pub total_leaves: usize,
    pub proof_steps: Vec<MerkleProofStep>,
}

impl MerkleInclusionProof {
    /// Verify this inclusion proof against a canonical leaf binding and root hash
    pub fn verify(&self, leaf: &MerkleLeafBinding, root: &[u8; 32]) -> Result<bool> {
        let mut current_hash = leaf.compute_leaf_hash();

        for step in &self.proof_steps {
            current_hash = if step.is_left {
                hash_merkle_node(&step.sibling_hash, &current_hash)
            } else {
                hash_merkle_node(&current_hash, &step.sibling_hash)
            };
        }

        if &current_hash == root {
            Ok(true)
        } else {
            Err(QShieldError::MerkleVerificationFailed(
                self.leaf_index,
                format!(
                    "Calculated root {:x?} does not match expected root {:x?}",
                    current_hash, root
                ),
            ))
        }
    }
}

/// Complete Merkle Tree holding all leaf nodes and internal layer nodes
#[derive(Debug, Clone)]
pub struct MerkleTree {
    leaf_bindings: Vec<MerkleLeafBinding>,
    layers: Vec<Vec<[u8; 32]>>,
}

impl MerkleTree {
    /// Build Merkle tree from slice of leaf bindings
    pub fn build(leaves: Vec<MerkleLeafBinding>) -> Result<Self> {
        if leaves.is_empty() {
            return Err(QShieldError::CryptoError("Cannot build Merkle tree with 0 leaves".into()));
        }

        let mut current_layer: Vec<[u8; 32]> = leaves
            .iter()
            .map(|leaf| leaf.compute_leaf_hash())
            .collect();

        let mut layers = Vec::new();
        layers.push(current_layer.clone());

        while current_layer.len() > 1 {
            let mut next_layer = Vec::new();
            let mut i = 0;
            while i < current_layer.len() {
                let left = current_layer[i];
                let right = if i + 1 < current_layer.len() {
                    current_layer[i + 1]
                } else {
                    current_layer[i] // Duplicate last node if odd
                };
                let parent = hash_merkle_node(&left, &right);
                next_layer.push(parent);
                i += 2;
            }
            layers.push(next_layer.clone());
            current_layer = next_layer;
        }

        Ok(Self {
            leaf_bindings: leaves,
            layers,
        })
    }

    /// Get root hash of the Merkle tree
    pub fn root_hash(&self) -> [u8; 32] {
        self.layers.last().expect("Tree must have at least 1 layer")[0]
    }

    /// Generate an inclusion proof for a specific leaf index
    pub fn generate_proof(&self, leaf_index: usize) -> Result<MerkleInclusionProof> {
        if leaf_index >= self.leaf_bindings.len() {
            return Err(QShieldError::CryptoError(format!(
                "Leaf index {} out of range (total: {})",
                leaf_index,
                self.leaf_bindings.len()
            )));
        }

        let mut steps = Vec::new();
        let mut idx = leaf_index;

        for layer in &self.layers[..self.layers.len() - 1] {
            let is_right_sibling = idx % 2 == 0;
            let sibling_idx = if is_right_sibling {
                if idx + 1 < layer.len() {
                    idx + 1
                } else {
                    idx // Self-paired odd node
                }
            } else {
                idx - 1
            };

            steps.push(MerkleProofStep {
                sibling_hash: layer[sibling_idx],
                is_left: !is_right_sibling,
            });

            idx /= 2;
        }

        Ok(MerkleInclusionProof {
            leaf_index,
            total_leaves: self.leaf_bindings.len(),
            proof_steps: steps,
        })
    }

    pub fn leaves(&self) -> &[MerkleLeafBinding] {
        &self.leaf_bindings
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_merkle_tree_root_and_inclusion_proof_verification() {
        let tid = TransferId::new_v4();
        let iid = ImageId::new("img_test_merkle");

        let mut leaves = Vec::new();
        for i in 0..7 {
            // Test odd number of leaves
            let coord = TileCoordinate {
                tile_id: TileId(i as u32),
                row: 0,
                col: i as u32,
                x: i as u32 * 64,
                y: 0,
                width: 64,
                height: 64,
            };
            let tile_hash = [i as u8; 32];
            leaves.push(MerkleLeafBinding::new(tid, &iid, &coord, tile_hash, "RGB8"));
        }

        let tree = MerkleTree::build(leaves.clone()).unwrap();
        let root = tree.root_hash();
        assert_ne!(root, [0u8; 32]);

        // Verify proofs for all leaves
        for (i, leaf) in leaves.iter().enumerate() {
            let proof = tree.generate_proof(i).unwrap();
            assert!(proof.verify(leaf, &root).unwrap());
        }

        // Test proof failure on tampered leaf data
        let mut tampered_leaf = leaves[0].clone();
        tampered_leaf.width = 128; // Tampered width
        let proof0 = tree.generate_proof(0).unwrap();
        assert!(proof0.verify(&tampered_leaf, &root).is_err());

        // Test proof failure on tampered tile hash
        let mut tampered_hash_leaf = leaves[1].clone();
        tampered_hash_leaf.tile_hash[0] ^= 0xFF;
        let proof1 = tree.generate_proof(1).unwrap();
        assert!(proof1.verify(&tampered_hash_leaf, &root).is_err());
    }
}
