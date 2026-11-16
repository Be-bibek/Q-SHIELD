//! State machine for reliable, resumable transfer session across intermittent links.

use crate::messages::{FragmentAckMessage, FragmentMessage, SessionInitMessage};
use qshield_core::{
    FragmentId, FragmentState, ImageId, TransferId,
};
use qshield_crypto::{DerivedKey, PqcCiphertext};
use qshield_fragment::EncryptedFragment;
use qshield_manifest::SignedManifest;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SessionStatus {
    Initializing,
    Transmitting,
    Interrupted,
    Completed,
    Failed,
}

/// Transmission session state manager at the sender side
pub struct SenderTransferSession {
    pub transfer_id: TransferId,
    pub image_id: ImageId,
    pub sender_id: String,
    pub receiver_id: String,
    pub session_key: DerivedKey,
    pub kem_ciphertext: PqcCiphertext,
    pub signed_manifest: SignedManifest,
    pub fragments: Vec<EncryptedFragment>,
    pub fragment_states: HashMap<FragmentId, FragmentState>,
    pub status: SessionStatus,
    pub acked_fragments: HashSet<FragmentId>,
    pub in_flight_fragments: HashSet<FragmentId>,
    pub transmitted_bytes: u64,
    pub retransmitted_bytes: u64,
    pub interruption_count: u32,
}

impl SenderTransferSession {
    pub fn new(
        transfer_id: TransferId,
        image_id: ImageId,
        sender_id: impl Into<String>,
        receiver_id: impl Into<String>,
        session_key: DerivedKey,
        kem_ciphertext: PqcCiphertext,
        signed_manifest: SignedManifest,
        fragments: Vec<EncryptedFragment>,
    ) -> Self {
        let mut fragment_states = HashMap::new();
        for frag in &fragments {
            fragment_states.insert(frag.header.fragment_id, FragmentState::Queued);
        }

        Self {
            transfer_id,
            image_id,
            sender_id: sender_id.into(),
            receiver_id: receiver_id.into(),
            session_key,
            kem_ciphertext,
            signed_manifest,
            fragments,
            fragment_states,
            status: SessionStatus::Initializing,
            acked_fragments: HashSet::new(),
            in_flight_fragments: HashSet::new(),
            transmitted_bytes: 0,
            retransmitted_bytes: 0,
            interruption_count: 0,
        }
    }

    /// Generate the initial handshake message containing KEM ciphertext and signed manifest
    pub fn create_init_message(&self) -> SessionInitMessage {
        SessionInitMessage {
            transfer_id: self.transfer_id,
            image_id: self.image_id.clone(),
            sender_id: self.sender_id.clone(),
            receiver_id: self.receiver_id.clone(),
            kem_ciphertext: self.kem_ciphertext.clone(),
            signed_manifest: self.signed_manifest.clone(),
        }
    }

    /// Mark session as active after receiver confirms handshake
    pub fn on_session_initialized(&mut self) {
        self.status = SessionStatus::Transmitting;
    }

    /// Retrieve the next batch of fragments ready for transmission
    pub fn next_fragments(&mut self, batch_size: usize) -> Vec<FragmentMessage> {
        if self.status != SessionStatus::Transmitting {
            return Vec::new();
        }

        let mut batch = Vec::new();
        for frag in &mut self.fragments {
            if batch.len() >= batch_size {
                break;
            }

            let fid = frag.header.fragment_id;
            let current_state = self.fragment_states.get(&fid).copied().unwrap_or(FragmentState::Queued);

            if current_state == FragmentState::Queued || current_state == FragmentState::Failed {
                let is_retransmission = current_state == FragmentState::Failed;
                let wire_len = frag.wire_size() as u64;

                if is_retransmission {
                    self.retransmitted_bytes += wire_len;
                } else {
                    self.transmitted_bytes += wire_len;
                }

                frag.state = FragmentState::Transmitted;
                self.fragment_states.insert(fid, FragmentState::Transmitted);
                self.in_flight_fragments.insert(fid);

                batch.push(FragmentMessage {
                    transfer_id: self.transfer_id,
                    fragment: frag.clone(),
                });
            }
        }

        batch
    }

    /// Process receiver acknowledgement
    pub fn process_ack(&mut self, ack: &FragmentAckMessage) {
        self.in_flight_fragments.remove(&ack.fragment_id);

        if ack.verified {
            self.acked_fragments.insert(ack.fragment_id);
            self.fragment_states.insert(ack.fragment_id, FragmentState::Verified);
        } else {
            // Negative acknowledgement: mark for retransmission
            self.fragment_states.insert(ack.fragment_id, FragmentState::Failed);
        }

        if self.acked_fragments.len() == self.fragments.len() {
            self.status = SessionStatus::Completed;
        }
    }

    /// Simulate loss of signal / contact interruption: freezes in-flight fragments and sets status to Interrupted
    pub fn handle_link_interruption(&mut self) {
        self.status = SessionStatus::Interrupted;
        self.interruption_count += 1;

        // In-flight unacknowledged fragments reset to Queued/Failed for retransmission
        for fid in self.in_flight_fragments.drain() {
            if !self.acked_fragments.contains(&fid) {
                self.fragment_states.insert(fid, FragmentState::Queued);
            }
        }
    }

    /// Resume transmission upon new contact window
    pub fn resume_link(&mut self) {
        if self.status == SessionStatus::Interrupted {
            self.status = SessionStatus::Transmitting;
        }
    }

    pub fn is_completed(&self) -> bool {
        self.status == SessionStatus::Completed || self.acked_fragments.len() == self.fragments.len()
    }

    pub fn progress_ratio(&self) -> f32 {
        if self.fragments.is_empty() {
            1.0
        } else {
            self.acked_fragments.len() as f32 / self.fragments.len() as f32
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use qshield_core::{ImageDimensions, ImageFormat, TileCoordinate, TileId};
    use qshield_crypto::PqcSigningKey;
    use qshield_fragment::{FragmentHeader, NONCE_SIZE};
    use qshield_manifest::ImageManifest;

    fn make_test_session() -> SenderTransferSession {
        let tid = TransferId::new_v4();
        let iid = ImageId::new("img_test_session");
        let dims = ImageDimensions {
            width: 128,
            height: 128,
            channels: 3,
            bit_depth: 8,
        };
        let coord = TileCoordinate {
            tile_id: TileId(0),
            row: 0,
            col: 0,
            x: 0,
            y: 0,
            width: 128,
            height: 128,
        };

        let manifest = ImageManifest::new(
            tid,
            iid.clone(),
            "SAT_01",
            "GS_01",
            dims,
            ImageFormat::Png,
            [0x11; 32],
            vec![coord],
            vec![[0x22; 32]],
            [0x33; 32],
            None,
            1.0,
        )
        .unwrap();

        let sk = PqcSigningKey::generate();
        let signed_manifest = manifest.sign(&sk);

        let key = DerivedKey::new([0x99; 32]);
        let kem_ct = PqcCiphertext {
            bytes: vec![0u8; 1088],
        };

        let mut fragments = Vec::new();
        for i in 0..5 {
            let header = FragmentHeader {
                transfer_id: tid,
                image_id: iid.clone(),
                fragment_id: FragmentId(i),
                tile_id: TileId(0),
                sequence_number: i,
                fragment_offset: i * 1000,
                fragment_length: 1000,
                total_expected_image_size: 5000,
                tile_data_total_len: 5000,
                is_last_fragment_of_tile: i == 4,
                is_last_fragment_of_image: i == 4,
                nonce: [i as u8; NONCE_SIZE],
                importance: 1.0,
            };
            let ciphertext = vec![0xEE; 1016];
            let hash = EncryptedFragment::compute_fragment_hash(&header, &ciphertext);
            fragments.push(EncryptedFragment {
                header,
                ciphertext_and_tag: ciphertext,
                fragment_hash: hash,
                merkle_proof: None,
                state: FragmentState::Created,
            });
        }

        SenderTransferSession::new(
            tid,
            iid,
            "SAT_01",
            "GS_01",
            key,
            kem_ct,
            signed_manifest,
            fragments,
        )
    }

    #[test]
    fn test_interrupted_transmission_and_resumption() {
        let mut session = make_test_session();
        session.on_session_initialized();

        // 1. Send first 2 fragments
        let batch1 = session.next_fragments(2);
        assert_eq!(batch1.len(), 2);
        assert_eq!(batch1[0].fragment.header.fragment_id, FragmentId(0));
        assert_eq!(batch1[1].fragment.header.fragment_id, FragmentId(1));

        // 2. Receiver acks fragment 0, but fragment 1 is in-flight when link drops
        session.process_ack(&FragmentAckMessage {
            transfer_id: session.transfer_id,
            fragment_id: FragmentId(0),
            verified: true,
            error_msg: None,
        });

        // 3. Link gets interrupted
        session.handle_link_interruption();
        assert_eq!(session.status, SessionStatus::Interrupted);

        // While interrupted, no fragments should be emitted
        let empty_batch = session.next_fragments(2);
        assert!(empty_batch.is_empty());

        // 4. Link resumes (next contact window)
        session.resume_link();
        assert_eq!(session.status, SessionStatus::Transmitting);

        // Fragment 1 was unacknowledged, so it should be retransmitted
        let batch2 = session.next_fragments(2);
        assert_eq!(batch2.len(), 2);
        assert_eq!(batch2[0].fragment.header.fragment_id, FragmentId(1));
        assert_eq!(batch2[1].fragment.header.fragment_id, FragmentId(2));

        // Ack remaining fragments
        for fid in [FragmentId(1), FragmentId(2), FragmentId(3), FragmentId(4)] {
            session.process_ack(&FragmentAckMessage {
                transfer_id: session.transfer_id,
                fragment_id: fid,
                verified: true,
                error_msg: None,
            });
        }

        assert!(session.is_completed());
        assert_eq!(session.progress_ratio(), 1.0);
    }
}
