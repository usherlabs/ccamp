use crate::{merkle::MerkleRoot, signature::{NotaryPublicKey, SignatureVerifyError}};
use mpz_core::serialize::CanonicalSerialize;
use mpz_garble_core::ChaChaEncoder;
use serde::{Deserialize, Serialize};
use crate::signature::Signature;

/// An error that can occur while verifying a [`SessionProof`].
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum SessionProofError {
    /// Session proof is missing Notary signature
    #[error("session proof is missing notary signature")]
    MissingNotarySignature,
    /// Invalid signature
    #[error(transparent)]
    InvalidSignature(#[from] SignatureVerifyError),
    /// Invalid server name.
    #[error("invalid server name: {0}")]
    InvalidServerName(String),
    /// Invalid handshake
    #[error("handshake verification failed: {0}")]
    InvalidHandshake(String),
    /// Invalid server certificate
    #[error("server certificate verification failed: {0}")]
    InvalidServerCertificate(String),
}

/// A session proof which is created from a [crate::session::NotarizedSession]
///
/// Proof of the TLS handshake, server identity, and commitments to the transcript.
#[derive(Debug, Serialize, Deserialize)]
pub struct SessionProof {
    /// The session header
    pub header: SessionHeader,
    // /// Signature for the session header, if the notary signed it
    pub signature: Option<Signature>,
    // /// Information about the server
    // pub session_info: SessionInfo,
}

/// An authentic session header from the Notary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionHeader {
    /// A PRG seeds used to generate encodings for the plaintext
    encoder_seed: [u8; 32],

    /// The root of the Merkle tree of all the commitments. The Prover must prove that each one of the
    /// `commitments` is included in the Merkle tree.
    /// This approach allows the Prover to hide from the Notary the exact amount of commitments thus
    /// increasing Prover privacy against the Notary.
    /// The root was made known to the Notary before the Notary opened his garbled circuits
    /// to the Prover.
    merkle_root: MerkleRoot,

    /// Bytelength of all data which was sent to the webserver
    sent_len: usize,
    /// Bytelength of all data which was received from the webserver
    recv_len: usize,
    // handshake_summary: HandshakeSummary,
}

impl SessionHeader {
    /// Create a new instance of SessionHeader
    pub fn new(
        encoder_seed: [u8; 32],
        merkle_root: MerkleRoot,
        sent_len: usize,
        recv_len: usize,
    ) -> Self {
        Self {
            encoder_seed,
            merkle_root,
            sent_len,
            recv_len,
        }
    }

    /// Create a new [ChaChaEncoder] from encoder_seed
    pub fn encoder(&self) -> ChaChaEncoder {
        ChaChaEncoder::new(self.encoder_seed)
    }

    /// Returns the seed used to generate plaintext encodings
    pub fn encoder_seed(&self) -> &[u8; 32] {
        &self.encoder_seed
    }

    /// Returns the merkle_root of the merkle tree of the prover's commitments
    pub fn merkle_root(&self) -> &MerkleRoot {
        &self.merkle_root
    }

    /// Returns the number of bytes sent to the server
    pub fn sent_len(&self) -> usize {
        self.sent_len
    }

    /// Returns the number of bytes received by the server
    pub fn recv_len(&self) -> usize {
        self.recv_len
    }
}

impl SessionProof {
    pub fn verify_test(&self, notary_pubkey : NotaryPublicKey) -> Result<(), SessionProofError> {
        let signature = self.signature.as_ref().ok_or(SessionProofError::MissingNotarySignature)?;
        signature.verify(&self.header.to_bytes(), notary_pubkey)?;
        Ok(())
    }
}