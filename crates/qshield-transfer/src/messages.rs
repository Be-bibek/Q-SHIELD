//! Protocol wire messages for session setup, fragment delivery, and acknowledgements.

use qshield_core::{FragmentId, ImageId, TransferId};
use qshield_crypto::PqcCiphertext;
use qshield_fragment::EncryptedFragment;
use qshield_manifest::SignedManifest;
use serde::{Deserialize, Serialize};

/// Session establishment message sent from transmitter (e.g. satellite) to receiver (e.g. ground station)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SessionInitMessage {
    pub transfer_id: TransferId,
    pub image_id: ImageId,
    pub sender_id: String,
    pub receiver_id: String,
    pub kem_ciphertext: PqcCiphertext,
    pub signed_manifest: SignedManifest,
}

/// Session establishment confirmation from receiver
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SessionInitAck {
    pub transfer_id: TransferId,
    pub accepted: bool,
    pub reason: String,
}

/// Transmission unit wrapper sent across simulated link
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FragmentMessage {
    pub transfer_id: TransferId,
    pub fragment: EncryptedFragment,
}

/// Receiver acknowledgement for received and verified fragment
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FragmentAckMessage {
    pub transfer_id: TransferId,
    pub fragment_id: FragmentId,
    pub verified: bool,
    pub error_msg: Option<String>,
}
