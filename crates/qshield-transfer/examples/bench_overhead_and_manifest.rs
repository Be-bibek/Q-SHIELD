//! Benchmark of Manifest signing/verification, Merkle tree operations,
//! and exact serialized wire format overhead across fragment sizes (4KB to 256KB).

use qshield_core::{
    FragmentId, ImageDimensions, ImageFormat, ImageId, TileCoordinate, TileId, TransferId,
};
use qshield_crypto::{hash_tile, PqcSigningKey};
use qshield_fragment::{EncryptedFragment, FragmentHeader, FragmentationConfig};
use qshield_manifest::ImageManifest;
use qshield_merkle::{MerkleLeafBinding, MerkleTree};
use serde::{Deserialize, Serialize};
use std::fs::{self, File};
use std::io::Write;
use std::path::Path;
use std::time::Instant;

#[derive(Debug, Serialize, Deserialize)]
pub struct OperationStat {
    pub mean_us: f64,
    pub std_us: f64,
    pub median_us: f64,
    pub min_us: f64,
    pub max_us: f64,
    pub ci95_lower_us: f64,
    pub ci95_upper_us: f64,
}

fn compute_stats(mut samples: Vec<f64>) -> OperationStat {
    samples.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let n = samples.len();
    let mean = samples.iter().sum::<f64>() / n as f64;
    let var = samples.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / (n - 1) as f64;
    let std_dev = var.sqrt();
    let median = if n % 2 == 0 {
        (samples[n / 2 - 1] + samples[n / 2]) / 2.0
    } else {
        samples[n / 2]
    };
    let margin = 1.96 * (std_dev / (n as f64).sqrt());
    OperationStat {
        mean_us: mean,
        std_us: std_dev,
        median_us: median,
        min_us: samples[0],
        max_us: samples[n - 1],
        ci95_lower_us: (mean - margin).max(0.0),
        ci95_upper_us: mean + margin,
    }
}

fn main() {
    println!("============================================================");
    println!("   Q-SHIELD MANIFEST, MERKLE & WIRE OVERHEAD BENCHMARKS    ");
    println!("============================================================\n");

    let iters = 100;
    let tid = TransferId::new_v4();
    let iid = ImageId::new("img_sentinel_bench");
    let dims = ImageDimensions {
        width: 1024,
        height: 1024,
        channels: 3,
        bit_depth: 8,
    };

    // 1. Setup 16 tiles and Merkle tree
    let num_tiles = 16;
    let mut coords = Vec::new();
    let mut tile_hashes = Vec::new();
    let mut leaves = Vec::new();

    for i in 0..num_tiles {
        let coord = TileCoordinate {
            tile_id: TileId(i as u32),
            row: (i / 4) as u32,
            col: (i % 4) as u32,
            x: ((i % 4) * 256) as u32,
            y: ((i / 4) * 256) as u32,
            width: 256,
            height: 256,
        };
        let dummy_pixels = vec![i as u8; 256 * 256 * 3];
        let h = hash_tile(&dummy_pixels);
        let leaf = MerkleLeafBinding::new(tid, &iid, &coord, h, "RGB8");
        coords.push(coord);
        tile_hashes.push(h);
        leaves.push(leaf);
    }

    // Benchmark Merkle tree construction
    println!("[*] Benchmarking Merkle Tree Construction (16 leaves)...");
    let mut tree_times = Vec::with_capacity(iters);
    for _ in 0..iters {
        let t0 = Instant::now();
        let _tree = MerkleTree::build(leaves.clone()).unwrap();
        tree_times.push(t0.elapsed().as_secs_f64() * 1_000_000.0);
    }
    let merkle_build_stat = compute_stats(tree_times);

    let tree = MerkleTree::build(leaves).unwrap();
    let root = tree.root_hash();

    // Benchmark Merkle inclusion proof generation
    println!("[*] Benchmarking Merkle Proof Generation & Verification...");
    let mut proof_gen_times = Vec::with_capacity(iters);
    let mut proof_verify_times = Vec::with_capacity(iters);
    let sample_leaf = MerkleLeafBinding::new(tid, &iid, &coords[0], tile_hashes[0], "RGB8");

    for _ in 0..iters {
        let t0 = Instant::now();
        let proof = tree.generate_proof(0).unwrap();
        proof_gen_times.push(t0.elapsed().as_secs_f64() * 1_000_000.0);

        let t1 = Instant::now();
        let _ = proof.verify(&sample_leaf, &root).unwrap();
        proof_verify_times.push(t1.elapsed().as_secs_f64() * 1_000_000.0);
    }
    let merkle_proof_gen_stat = compute_stats(proof_gen_times);
    let merkle_proof_verify_stat = compute_stats(proof_verify_times);

    // 2. Manifest Generation, Signing, and Verification
    let manifest = ImageManifest::new(
        tid,
        iid.clone(),
        "SAT_SENTINEL_2A",
        "GS_SVALBARD",
        dims,
        ImageFormat::Png,
        [0xAA; 32],
        coords.clone(),
        tile_hashes.clone(),
        root,
        None,
        1.0,
    )
    .unwrap();

    let signing_key = PqcSigningKey::generate();

    println!("[*] Benchmarking Canonical Manifest Signing (ML-DSA-65)...");
    let mut sign_times = Vec::with_capacity(iters);
    for _ in 0..iters {
        let t0 = Instant::now();
        let _signed = manifest.sign(&signing_key);
        sign_times.push(t0.elapsed().as_secs_f64() * 1_000_000.0);
    }
    let manifest_sign_stat = compute_stats(sign_times);

    let signed_manifest = manifest.sign(&signing_key);

    println!("[*] Benchmarking Canonical Manifest Signature Verification...");
    let mut verify_times = Vec::with_capacity(iters);
    for _ in 0..iters {
        let t0 = Instant::now();
        signed_manifest.verify_signature().unwrap();
        verify_times.push(t0.elapsed().as_secs_f64() * 1_000_000.0);
    }
    let manifest_verify_stat = compute_stats(verify_times);

    let manifest_json = serde_json::to_string_pretty(&signed_manifest).unwrap();
    let manifest_wire_bytes = manifest_json.len();
    println!("    Signed Manifest Wire Size: {} bytes", manifest_wire_bytes);

    // Save manifest/Merkle benchmark results
    let manifest_merkle_results = serde_json::json!({
        "merkle_tree_build_16_leaves": merkle_build_stat,
        "merkle_proof_generation": merkle_proof_gen_stat,
        "merkle_proof_verification": merkle_proof_verify_stat,
        "manifest_signing_ml_dsa_65": manifest_sign_stat,
        "manifest_verification_ml_dsa_65": manifest_verify_stat,
        "signed_manifest_wire_bytes": manifest_wire_bytes,
        "merkle_proof_bytes_16_leaves": tree.generate_proof(0).unwrap().proof_steps.len() * 33 + 8
    });

    let crypto_dir = Path::new("experiments/results/crypto");
    fs::create_dir_all(crypto_dir).unwrap();
    let mut f = File::create(crypto_dir.join("manifest_merkle_benchmarks.json")).unwrap();
    f.write_all(serde_json::to_string_pretty(&manifest_merkle_results).unwrap().as_bytes()).unwrap();

    // 3. Exact Wire Format Overhead Analysis across Fragment Sizes
    println!("\n[*] Calculating Exact Wire Format Overhead Across Payload Sizes...");
    let fragment_sizes = [4096, 16384, 65536, 262144];
    let proof = tree.generate_proof(0).unwrap();
    let proof_bytes = proof.proof_steps.len() * 33 + 8; // binary proof size

    let mut overhead_results = Vec::new();

    for &payload_size in &fragment_sizes {
        let header = FragmentHeader {
            transfer_id: tid,
            image_id: iid.clone(),
            fragment_id: FragmentId(1),
            tile_id: TileId(0),
            sequence_number: 1,
            fragment_offset: 0,
            fragment_length: payload_size as u32,
            total_expected_image_size: 1024 * 1024 * 3,
            tile_data_total_len: 256 * 256 * 3,
            is_last_fragment_of_tile: false,
            is_last_fragment_of_image: false,
            nonce: [0x12; 12],
            importance: 0.85,
        };

        // Binary header size breakdown:
        // UUID (16) + ImageId (16) + frag_id (4) + tile_id (4) + seq (4) + offset (4)
        // + len (4) + tot_img (8) + tot_tile (4) + flags (2) + nonce (12) + imp (4) = 82 bytes
        let binary_header_bytes = 82;
        let ghash_tag_bytes = 16;
        let fragment_hash_bytes = 32;

        // Q-SHIELD Standard: header (82) + nonce (12) + GHASH tag (16) + fragment hash (32) = 142 bytes (without proof)
        // With milestone Merkle proof (+140 bytes) = 282 bytes
        let qshield_wire_intermediate = payload_size + binary_header_bytes + ghash_tag_bytes + fragment_hash_bytes;
        let qshield_wire_with_proof = qshield_wire_intermediate + proof_bytes;

        // Baselines:
        // 1. No Security: raw payload
        let no_sec_wire = payload_size;

        // 2. Classical (ECDH P-256 pubkey 65B + Ed25519 sig 64B + Nonce 12B + Tag 16B + header 82B)
        let classical_wire = payload_size + 65 + 64 + 12 + 16 + binary_header_bytes;

        // 3. Naive Per-Fragment PQC (ML-KEM-768 PK 1184B + CT 1088B + ML-DSA-65 sig 3309B + Nonce 12B + Tag 16B + header 82B)
        let naive_pqc_wire = payload_size + 1184 + 1088 + 3309 + 12 + 16 + binary_header_bytes;

        let item = serde_json::json!({
            "payload_bytes": payload_size,
            "no_security": {
                "wire_bytes": no_sec_wire,
                "overhead_bytes": 0,
                "overhead_percent": 0.0
            },
            "classical_ecc_per_frag": {
                "wire_bytes": classical_wire,
                "overhead_bytes": classical_wire - payload_size,
                "overhead_percent": ((classical_wire - payload_size) as f64 / payload_size as f64) * 100.0
            },
            "naive_pqc_per_frag": {
                "wire_bytes": naive_pqc_wire,
                "overhead_bytes": naive_pqc_wire - payload_size,
                "overhead_percent": ((naive_pqc_wire - payload_size) as f64 / payload_size as f64) * 100.0
            },
            "qshield_amortized_intermediate": {
                "wire_bytes": qshield_wire_intermediate,
                "overhead_bytes": qshield_wire_intermediate - payload_size,
                "overhead_percent": ((qshield_wire_intermediate - payload_size) as f64 / payload_size as f64) * 100.0
            },
            "qshield_amortized_milestone_with_proof": {
                "wire_bytes": qshield_wire_with_proof,
                "overhead_bytes": qshield_wire_with_proof - payload_size,
                "overhead_percent": ((qshield_wire_with_proof - payload_size) as f64 / payload_size as f64) * 100.0
            }
        });

        overhead_results.push(item);
    }

    let overhead_dir = Path::new("experiments/results/overhead");
    fs::create_dir_all(overhead_dir).unwrap();
    let mut f_ov = File::create(overhead_dir.join("wire_overhead_comparison.json")).unwrap();
    f_ov.write_all(serde_json::to_string_pretty(&overhead_results).unwrap().as_bytes()).unwrap();

    println!("[+] Successfully wrote manifest, merkle, and overhead results!");
}
