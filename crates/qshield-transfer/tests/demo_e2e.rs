//! End-to-End Working Demonstration of Q-SHIELD
//!
//! Validates:
//! 1. Real Image Loading (datasets/satellite/sentinel2_sample.png)
//! 2. Image Preprocessing and Tiling (256x256)
//! 3. Merkle Tree Construction and Inclusion Proof Generation
//! 4. Canonical Manifest Generation and ML-DSA-65 Signing
//! 5. ML-KEM-768 Post-Quantum Key Establishment
//! 6. Context-Bound HKDF-SHA-256 AES-256-GCM Session Key Derivation
//! 7. Tile AEAD Encryption with Guaranteed Nonce Uniqueness
//! 8. Simulated LEO Link Interruption (AOS -> LOS)
//! 9. Transmission Resumption on Next Contact Window
//! 10. Receiver Authentication, Cryptographic Verification (Sig, AEAD, Hash, Merkle)
//! 11. Complete Progressive Lossless Image Reconstruction Exact Match Verification

use qshield_core::{ImageId, Result, TransferId};
use qshield_crypto::{
    derive_session_key, encapsulate, sha256_digest, KeyDerivationContext,
    NonceManager, PqcSigningKey,
};
use qshield_fragment::{fragment_and_encrypt_tile, FragmentationConfig};
use qshield_image::{decode_image, slice_image, TilerConfig};
use qshield_manifest::ImageManifest;
use qshield_merkle::{MerkleLeafBinding, MerkleTree};
use qshield_receiver::VerifiableReceiverSession;
use qshield_transfer::{FragmentAckMessage, SenderTransferSession};
use std::fs;
use std::path::Path;

#[test]
fn test_qshield_end_to_end_demonstration() -> Result<()> {
    println!("\n=======================================================");
    println!("   Q-SHIELD END-TO-END DEMONSTRATION & VERIFICATION    ");
    println!("=======================================================\n");

    // ---------------------------------------------------------
    // STAGE A: Load Real Image and Preprocess
    // ---------------------------------------------------------
    let img_path = Path::new("../../datasets/satellite/sentinel2_sample.png");
    let fallback_path = Path::new("datasets/satellite/sentinel2_sample.png");
    let target_path = if img_path.exists() { img_path } else { fallback_path };

    let image_bytes = fs::read(target_path).expect("Failed to read sample satellite image");
    let (raw_pixels, dimensions, format) = decode_image(&image_bytes)?;

    println!(
        "[1] Loaded image: {}x{}, channels={}, total raw bytes={}",
        dimensions.width,
        dimensions.height,
        dimensions.channels,
        raw_pixels.len()
    );

    let tiler_cfg = TilerConfig {
        tile_width: 256,
        tile_height: 256,
    };
    let tiles = slice_image(&raw_pixels, &dimensions, &tiler_cfg)?;
    println!("[2] Sliced image into {} grid tiles (256x256)", tiles.len());

    // ---------------------------------------------------------
    // STAGE B: Merkle Tree & Canonical Manifest Creation
    // ---------------------------------------------------------
    let transfer_id = TransferId::new_v4();
    let image_id = ImageId::new("SENTINEL2_2026_TEST");
    let orig_hash = sha256_digest(&raw_pixels);

    let mut tile_coords = Vec::new();
    let mut tile_hashes = Vec::new();
    let mut merkle_leaves = Vec::new();

    for tile in &tiles {
        tile_coords.push(tile.coordinate);
        tile_hashes.push(tile.hash);

        merkle_leaves.push(MerkleLeafBinding::new(
            transfer_id,
            &image_id,
            &tile.coordinate,
            tile.hash,
            "RGB8",
        ));
    }

    let merkle_tree = MerkleTree::build(merkle_leaves)?;
    let merkle_root = merkle_tree.root_hash();
    println!("[3] Computed Merkle root: {:x?}", &merkle_root[..8]);

    let manifest = ImageManifest::new(
        transfer_id,
        image_id.clone(),
        "SENTINEL-2A",
        "GS-SVALBARD",
        dimensions,
        format,
        orig_hash,
        tile_coords,
        tile_hashes,
        merkle_root,
        Some("WM-PROV-SENTINEL-01".into()),
        0.95,
    )?;

    // ---------------------------------------------------------
    // STAGE C: Post-Quantum Digital Signature (ML-DSA-65)
    // ---------------------------------------------------------
    let sat_signing_key = PqcSigningKey::generate();
    let signed_manifest = manifest.sign(&sat_signing_key);
    println!("[4] Manifest signed with ML-DSA-65 (sig length={} bytes)", signed_manifest.signature.bytes.len());

    // ---------------------------------------------------------
    // STAGE D: Post-Quantum Key Establishment (ML-KEM-768)
    // ---------------------------------------------------------
    let mut receiver_session = VerifiableReceiverSession::new("GS-SVALBARD");
    let ground_station_pk = receiver_session.public_key();

    let (kem_ciphertext, sat_shared_secret) = encapsulate(&ground_station_pk)?;
    println!("[5] Established ML-KEM-768 shared secret (ciphertext={} bytes)", kem_ciphertext.bytes.len());

    // ---------------------------------------------------------
    // STAGE E: HKDF-SHA-256 Session Key Derivation
    // ---------------------------------------------------------
    let sat_ctx = KeyDerivationContext::standard("SENTINEL-2A", "GS-SVALBARD", transfer_id);
    let session_key = derive_session_key(sat_shared_secret.as_bytes(), &sat_ctx);
    println!("[6] Derived AES-256-GCM symmetric session key via HKDF-SHA-256 with contextual binding");

    // ---------------------------------------------------------
    // STAGE F: Tile Encryption and Fragmentation
    // ---------------------------------------------------------
    let nonce_mgr = NonceManager::new([0xCA, 0xFE, 0xBA, 0xBE]);
    let mut seq = 0u32;
    let mut frag_counter = 0u32;
    let frag_cfg = FragmentationConfig {
        max_fragment_payload_size: 64 * 1024, // 64 KB fragments
    };

    let mut all_fragments = Vec::new();
    for (tile_idx, tile) in tiles.iter().enumerate() {
        let proof = merkle_tree.generate_proof(tile_idx)?;
        let frags = fragment_and_encrypt_tile(
            tile,
            transfer_id,
            &image_id,
            raw_pixels.len() as u64,
            &mut seq,
            &mut frag_counter,
            &session_key,
            &nonce_mgr,
            Some(proof),
            &frag_cfg,
        )?;
        all_fragments.extend(frags);
    }
    println!("[7] Generated {} independently authenticated AEAD encrypted fragments", all_fragments.len());

    // ---------------------------------------------------------
    // STAGE G: Receiver Handshake & Session Initialization
    // ---------------------------------------------------------
    receiver_session.init_session(&kem_ciphertext, &signed_manifest)?;
    println!("[8] Ground Station decapsulated shared secret and verified ML-DSA-65 manifest signature");

    let mut sender_session = SenderTransferSession::new(
        transfer_id,
        image_id.clone(),
        "SENTINEL-2A",
        "GS-SVALBARD",
        session_key,
        kem_ciphertext,
        signed_manifest,
        all_fragments,
    );
    sender_session.on_session_initialized();

    // ---------------------------------------------------------
    // STAGE H: Transmit First Batch over Simulated LEO Link
    // ---------------------------------------------------------
    let batch1 = sender_session.next_fragments(4); // deliver first 4 fragments
    println!("[9] Transmitting Batch 1: {} fragments", batch1.len());

    for msg in batch1 {
        let _verified = receiver_session.process_fragment(&msg.fragment)?;
        sender_session.process_ack(&FragmentAckMessage {
            transfer_id,
            fragment_id: msg.fragment.header.fragment_id,
            verified: true,
            error_msg: None,
        });
    }

    let progress1 = receiver_session.verified_tile_ratio();
    println!("[10] Batch 1 delivery complete. Verified tile ratio: {:.1}%", progress1 * 100.0);

    // ---------------------------------------------------------
    // STAGE I: Simulate Interrupted Satellite Transmission (LOS)
    // ---------------------------------------------------------
    println!("[11] Simulated contact interruption: Satellite passed out of view of Svalbard Station (LOS)");
    sender_session.handle_link_interruption();
    assert_eq!(sender_session.status, qshield_transfer::SessionStatus::Interrupted);

    // ---------------------------------------------------------
    // STAGE J: Simulate Next Contact Window (AOS) & Resumption
    // ---------------------------------------------------------
    println!("[12] Simulated contact resumption: Satellite in view for pass 2 (AOS)");
    sender_session.resume_link();

    while !sender_session.is_completed() {
        let batch = sender_session.next_fragments(4);
        if batch.is_empty() {
            break;
        }
        for msg in batch {
            let _ = receiver_session.process_fragment(&msg.fragment)?;
            sender_session.process_ack(&FragmentAckMessage {
                transfer_id,
                fragment_id: msg.fragment.header.fragment_id,
                verified: true,
                error_msg: None,
            });
        }
    }

    assert!(sender_session.is_completed());
    println!("[13] Resumed transfer completed. 100% fragments acknowledged.");

    // ---------------------------------------------------------
    // STAGE K: Verified Image Reconstruction and Lossless Match
    // ---------------------------------------------------------
    let (reconstructed_pixels, is_complete) = receiver_session.render_reconstruction([0, 0, 0])?;
    assert!(is_complete, "Reconstruction must be complete");
    assert_eq!(
        raw_pixels.len(),
        reconstructed_pixels.len(),
        "Reconstructed byte length must match original"
    );

    // Bit-for-bit exact match verification
    assert_eq!(
        raw_pixels,
        reconstructed_pixels,
        "CRITICAL: Reconstructed pixels must be bit-for-bit identical to original!"
    );

    println!("[14] SUCCESS: Reconstructed image verified bit-identical to original!");
    println!("=======================================================\n");

    Ok(())
}
