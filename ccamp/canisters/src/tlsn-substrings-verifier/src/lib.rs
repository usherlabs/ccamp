pub mod commitment;
pub mod merkle;
pub mod proof;
pub mod transcript;
pub mod signature;

pub use transcript::{Direction, RedactedTranscript, Transcript, TranscriptSlice};

use mpz_garble_core::{encoding_state, EncodedValue};
use std::str;

/// A provider of plaintext encodings.
pub(crate) type EncodingProvider =
    Box<dyn Fn(&[&str]) -> Option<Vec<EncodedValue<encoding_state::Active>>> + Send>;

/// The maximum allowed total bytelength of all committed data. Used to prevent DoS during verification.
/// (this will cause the verifier to hash up to a max of 1GB * 128 = 128GB of plaintext encodings if the
/// commitment type is [crate::commitment::Blake3]).
///
/// This value must not exceed bcs's MAX_SEQUENCE_LENGTH limit (which is (1 << 31) - 1 by default)
const MAX_TOTAL_COMMITTED_DATA: usize = 1_000_000_000;

/// The encoding id
///
/// A 64 bit Blake3 hash which is used for the plaintext encodings
#[derive(Debug, Clone, Copy, PartialEq, Eq, Ord, PartialOrd, Hash)]
pub(crate) struct EncodingId(u64);

impl EncodingId {
    /// Create a new encoding ID.
    pub(crate) fn new(id: &str) -> Self {
        let hash = mpz_core::utils::blake3(id.as_bytes());
        Self(u64::from_be_bytes(hash[..8].try_into().unwrap()))
    }

    /// Returns the encoding ID.
    pub(crate) fn to_inner(self) -> u64 {
        self.0
    }
}
