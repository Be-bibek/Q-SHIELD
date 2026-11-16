//! Real Cryptographic Performance Benchmarking for Q-SHIELD.
//! Measures ML-KEM-768, ML-DSA-65, and AES-256-GCM latencies, throughput, and overhead.

use qshield_core::TransferId;
use qshield_crypto::{
    decrypt_aead, derive_session_key, encapsulate, encrypt_aead, KeyDerivationContext,
    NonceManager, PqcKeyPair, PqcSigningKey,
};
use serde_json::json;
use std::fs;
use std::time::Instant;

#[test]
fn benchmark_cryptography() {
    println!("\n--- RUNNING Q-SHIELD CRYPTOGRAPHIC BENCHMARKS ---");

    let iterations = 200;

    // 1. ML-KEM-768 Key Generation
    let start = Instant::now();
    for _ in 0..iterations {
        let _ = PqcKeyPair::generate();
    }
    let kem_keygen_us = start.elapsed().as_micros() as f64 / iterations as f64;

    // 2. ML-KEM-768 Encapsulation
    let kp = PqcKeyPair::generate();
    let pk = kp.public_key();
    let start = Instant::now();
    for _ in 0..iterations {
        let _ = encapsulate(&pk).unwrap();
    }
    let kem_encaps_us = start.elapsed().as_micros() as f64 / iterations as f64;

    // 3. ML-KEM-768 Decapsulation
    let (ct, _) = encapsulate(&pk).unwrap();
    let start = Instant::now();
    for _ in 0..iterations {
        let _ = kp.decapsulate(&ct).unwrap();
    }
    let kem_decaps_us = start.elapsed().as_micros() as f64 / iterations as f64;

    // 4. ML-DSA-65 Key Generation
    let start = Instant::now();
    for _ in 0..iterations {
        let _ = PqcSigningKey::generate();
    }
    let dsa_keygen_us = start.elapsed().as_micros() as f64 / iterations as f64;

    // 5. ML-DSA-65 Signing
    let sk = PqcSigningKey::generate();
    let vk = sk.verifying_key();
    let test_digest = [0x5Au8; 32];
    let start = Instant::now();
    for _ in 0..iterations {
        let _ = sk.sign(&test_digest);
    }
    let dsa_sign_us = start.elapsed().as_micros() as f64 / iterations as f64;

    // 6. ML-DSA-65 Verification
    let sig = sk.sign(&test_digest);
    let start = Instant::now();
    for _ in 0..iterations {
        let _ = vk.verify(&test_digest, &sig).unwrap();
    }
    let dsa_verify_us = start.elapsed().as_micros() as f64 / iterations as f64;

    // 7. AES-256-GCM Encryption Throughput (64 KB satellite tile fragments)
    let payload_size = 64 * 1024;
    let plaintext = vec![0x42u8; payload_size];
    let key = [0x77u8; 32];
    let nonce = [0x11u8; 12];
    let aad = b"TransferId:bench";

    let gcm_iters = 500;
    let start = Instant::now();
    for _ in 0..gcm_iters {
        let _ = encrypt_aead(&key, &nonce, &plaintext, aad).unwrap();
    }
    let enc_duration = start.elapsed().as_secs_f64();
    let total_mb = (payload_size * gcm_iters) as f64 / (1024.0 * 1024.0);
    let aes_enc_throughput_mb_s = total_mb / enc_duration;

    // 8. AES-256-GCM Decryption Throughput
    let ciphertext = encrypt_aead(&key, &nonce, &plaintext, aad).unwrap();
    let start = Instant::now();
    for _ in 0..gcm_iters {
        let _ = decrypt_aead(&key, &nonce, &ciphertext, aad).unwrap();
    }
    let dec_duration = start.elapsed().as_secs_f64();
    let aes_dec_throughput_mb_s = total_mb / dec_duration;

    println!("ML-KEM-768 KeyGen: {:.2} us", kem_keygen_us);
    println!("ML-KEM-768 Encaps: {:.2} us", kem_encaps_us);
    println!("ML-KEM-768 Decaps: {:.2} us", kem_decaps_us);
    println!("ML-DSA-65 KeyGen:  {:.2} us", dsa_keygen_us);
    println!("ML-DSA-65 Sign:    {:.2} us", dsa_sign_us);
    println!("ML-DSA-65 Verify:  {:.2} us", dsa_verify_us);
    println!("AES-256-GCM Enc Throughput: {:.2} MB/s", aes_enc_throughput_mb_s);
    println!("AES-256-GCM Dec Throughput: {:.2} MB/s", aes_dec_throughput_mb_s);

    let output_data = json!({
        "ml_kem_768": {
            "keygen_us": kem_keygen_us,
            "encaps_us": kem_encaps_us,
            "decaps_us": kem_decaps_us,
            "public_key_bytes": pk.bytes.len(),
            "ciphertext_bytes": ct.bytes.len(),
        },
        "ml_dsa_65": {
            "keygen_us": dsa_keygen_us,
            "sign_us": dsa_sign_us,
            "verify_us": dsa_verify_us,
            "public_key_bytes": vk.bytes.len(),
            "signature_bytes": sig.bytes.len(),
        },
        "aes_256_gcm": {
            "enc_throughput_mb_s": aes_enc_throughput_mb_s,
            "dec_throughput_mb_s": aes_dec_throughput_mb_s,
            "tag_size_bytes": 16,
            "nonce_size_bytes": 12,
        }
    });

    let _ = fs::create_dir_all("../../experiments/results");
    let _ = fs::write(
        "../../experiments/results/crypto_benchmarks.json",
        serde_json::to_string_pretty(&output_data).unwrap(),
    );
    let _ = fs::write(
        "experiments/results/crypto_benchmarks.json",
        serde_json::to_string_pretty(&output_data).unwrap(),
    );
}
