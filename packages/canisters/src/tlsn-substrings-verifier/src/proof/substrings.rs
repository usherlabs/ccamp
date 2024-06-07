#![allow(missing_docs)]
use crate::{
    commitment::{
        Commitment, CommitmentId, CommitmentInfo, CommitmentKind, CommitmentOpening,
        TranscriptCommitments,
    },
    merkle::MerkleProof,
    transcript::get_value_ids,
    Direction, EncodingId, RedactedTranscript, Transcript, TranscriptSlice,
    MAX_TOTAL_COMMITTED_DATA,
};

use mpz_circuits::types::ValueType;
use mpz_core::hash::Hash;
use mpz_garble_core::{encoding, encoding_state::Full, EncodedValue, Encoder};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use utils::range::{RangeDisjoint, RangeSet, RangeUnion, ToRangeSet};

use super::SessionHeader;

/// An error for [`SubstringsProofBuilder`]
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum SubstringsProofBuilderError {
    /// Invalid commitment id.
    #[error("invalid commitment id: {0:?}")]
    InvalidCommitmentId(CommitmentId),
    /// Missing commitment.
    #[error("missing commitment")]
    MissingCommitment,
    /// Invalid commitment type.
    #[error("commitment {0:?} is not a substrings commitment")]
    InvalidCommitmentType(CommitmentId),
    /// Attempted to add a commitment with a duplicate id.
    #[error("commitment with id {0:?} already exists")]
    DuplicateCommitmentId(CommitmentId),
}

/// A builder for [`SubstringsProof`]
pub struct SubstringsProofBuilder<'a> {
    commitments: &'a TranscriptCommitments,
    transcript_tx: &'a Transcript,
    transcript_rx: &'a Transcript,
    openings: HashMap<CommitmentId, (CommitmentInfo, CommitmentOpening)>,
}

opaque_debug::implement!(SubstringsProofBuilder<'_>);

impl<'a> SubstringsProofBuilder<'a> {
    /// Creates a new builder.
    pub fn new(
        commitments: &'a TranscriptCommitments,
        transcript_tx: &'a Transcript,
        transcript_rx: &'a Transcript,
    ) -> Self {
        Self {
            commitments,
            transcript_tx,
            transcript_rx,
            openings: HashMap::default(),
        }
    }

    /// Returns a reference to the commitments.
    pub fn commitments(&self) -> &TranscriptCommitments {
        self.commitments
    }

    /// Reveals data corresponding to the provided ranges in the sent direction.
    pub fn reveal_sent(
        &mut self,
        ranges: &dyn ToRangeSet<usize>,
        commitment_kind: CommitmentKind,
    ) -> Result<&mut Self, SubstringsProofBuilderError> {
        self.reveal(ranges, Direction::Sent, commitment_kind)
    }

    /// Reveals data corresponding to the provided transcript subsequence in the received direction.
    pub fn reveal_recv(
        &mut self,
        ranges: &dyn ToRangeSet<usize>,
        commitment_kind: CommitmentKind,
    ) -> Result<&mut Self, SubstringsProofBuilderError> {
        self.reveal(ranges, Direction::Received, commitment_kind)
    }

    /// Reveals data corresponding to the provided ranges and direction.
    pub fn reveal(
        &mut self,
        ranges: &dyn ToRangeSet<usize>,
        direction: Direction,
        commitment_kind: CommitmentKind,
    ) -> Result<&mut Self, SubstringsProofBuilderError> {
        let com = self
            .commitments
            .get_id_by_info(commitment_kind, &ranges.to_range_set(), direction)
            .ok_or(SubstringsProofBuilderError::MissingCommitment)?;

        self.reveal_by_id(com)
    }

    /// Reveals data corresponding to the provided commitment id
    pub fn reveal_by_id(
        &mut self,
        id: CommitmentId,
    ) -> Result<&mut Self, SubstringsProofBuilderError> {
        let commitment = self
            .commitments()
            .get(&id)
            .ok_or(SubstringsProofBuilderError::InvalidCommitmentId(id))?;

        let info = self
            .commitments()
            .get_info(&id)
            .expect("info exists if commitment exists");

        #[allow(irrefutable_let_patterns)]
        let Commitment::Blake3(commitment) = commitment
        else {
            return Err(SubstringsProofBuilderError::InvalidCommitmentType(id));
        };

        let transcript = match info.direction() {
            Direction::Sent => self.transcript_tx,
            Direction::Received => self.transcript_rx,
        };

        let data = transcript.get_bytes_in_ranges(info.ranges());

        // add commitment to openings and return an error if it is already present
        if self
            .openings
            .insert(id, (info.clone(), commitment.open(data).into()))
            .is_some()
        {
            return Err(SubstringsProofBuilderError::DuplicateCommitmentId(id));
        }

        Ok(self)
    }

    /// Builds the [`SubstringsProof`]
    pub fn build(self) -> Result<SubstringsProof, SubstringsProofBuilderError> {
        let Self {
            commitments,
            openings,
            ..
        } = self;

        let mut indices = openings
            .keys()
            .map(|id| id.to_inner() as usize)
            .collect::<Vec<_>>();
        indices.sort();

        let inclusion_proof = commitments.merkle_tree().proof(&indices);

        Ok(SubstringsProof {
            openings,
            inclusion_proof,
        })
    }
}

/// An error relating to [`SubstringsProof`]
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum SubstringsProofError {
    /// The proof contains more data than the maximum allowed.
    #[error(
        "substrings proof opens more data than the maximum allowed: {0} > {}",
        MAX_TOTAL_COMMITTED_DATA
    )]
    MaxDataExceeded(usize),
    /// The proof contains duplicate transcript data.
    #[error("proof contains duplicate transcript data")]
    DuplicateData(Direction, RangeSet<usize>),
    /// Range of the opening is out of bounds.
    #[error("range of opening {0:?} is out of bounds: {1}")]
    RangeOutOfBounds(CommitmentId, usize),
    /// The proof contains an invalid commitment opening.
    #[error("invalid opening for commitment id: {0:?}")]
    InvalidOpening(CommitmentId),
    /// The proof contains an invalid inclusion proof.
    #[error("invalid inclusion proof: {0}")]
    InvalidInclusionProof(String),
}

/// A substring proof using commitments
///
/// This substring proof contains the commitment openings and a proof
/// that the corresponding commitments are present in the merkle tree.
#[derive(Serialize, Deserialize, Clone)]
pub struct SubstringsProof {
    pub openings: HashMap<CommitmentId, (CommitmentInfo, CommitmentOpening)>,
    pub inclusion_proof: MerkleProof,
}

opaque_debug::implement!(SubstringsProof);

impl SubstringsProof {
    pub fn verify_with_precompute(
        self,
        header: &SessionHeader,
        encodings_list: HashMap<CommitmentId, Vec<EncodedValue<Full>>>,
    ) -> Result<(RedactedTranscript, RedactedTranscript), SubstringsProofError> {
        let Self {
            openings,
            inclusion_proof,
        } = self;

        let mut indices: Vec<usize> = Vec::with_capacity(openings.len());
        let mut expected_hashes: Vec<Hash> = Vec::with_capacity(openings.len());
        let mut sent = vec![0u8; header.sent_len()];
        let mut recv = vec![0u8; header.recv_len()];
        let mut sent_ranges = RangeSet::default();
        let mut recv_ranges = RangeSet::default();

        for (id, (info, opening)) in openings {
            // // Compute the expected hash of the commitment to make sure it is
            // // present in the merkle tree.
            indices.push(id.to_inner() as usize);
            expected_hashes.push(opening.recover(&encodings_list.get(&id).unwrap()).hash());
        }

        // Verify that the expected hashes are present in the merkle tree.
        //
        // This proves the Prover committed to the purported data prior to the encoder
        // seed being revealed.
        inclusion_proof
            .verify(header.merkle_root(), &indices, &expected_hashes)
            .map_err(|e| SubstringsProofError::InvalidInclusionProof(e.to_string()))?;

        // Iterate over the unioned ranges and create TranscriptSlices for each.
        // This ensures that the slices are sorted and disjoint.
        let sent_slices = sent_ranges
            .iter_ranges()
            .map(|range| TranscriptSlice::new(range.clone(), sent[range].to_vec()))
            .collect();
        let recv_slices = recv_ranges
            .iter_ranges()
            .map(|range| TranscriptSlice::new(range.clone(), recv[range].to_vec()))
            .collect();

        Ok((
            RedactedTranscript::new(header.sent_len(), sent_slices),
            RedactedTranscript::new(header.recv_len(), recv_slices),
        ))
    }

    pub fn extract_random_values(
        self,
        header: &SessionHeader,
    ) -> Result<(HashMap<CommitmentId, Vec<EncodedValue<Full>>>), SubstringsProofError> {
        let Self {
            openings,
            inclusion_proof,
        } = self;

        let mut indices: Vec<usize> = Vec::with_capacity(openings.len());
        let mut expected_hashes: Vec<Hash> = Vec::with_capacity(openings.len());
        let mut sent = vec![0u8; header.sent_len()];
        let mut recv = vec![0u8; header.recv_len()];
        let mut sent_ranges = RangeSet::default();
        let mut recv_ranges = RangeSet::default();
        let mut total_opened = 0u128;
        let mut encoding_list: HashMap<CommitmentId, Vec<EncodedValue<Full>>> = HashMap::new();

        for (id, (info, opening)) in openings {
            let CommitmentInfo {
                ranges, direction, ..
            } = info;

            let opened_len = ranges.len();

            // Make sure the amount of data being proved is bounded.
            total_opened += opened_len as u128;
            if total_opened > MAX_TOTAL_COMMITTED_DATA as u128 {
                return Err(SubstringsProofError::MaxDataExceeded(total_opened as usize));
            }

            // Make sure the opening length matches the ranges length.
            if opening.data().len() != opened_len {
                return Err(SubstringsProofError::InvalidOpening(id));
            }

            // Make sure duplicate data is not opened.
            match direction {
                Direction::Sent => {
                    if !sent_ranges.is_disjoint(&ranges) {
                        return Err(SubstringsProofError::DuplicateData(direction, ranges));
                    }
                    sent_ranges = sent_ranges.union(&ranges);
                }
                Direction::Received => {
                    if !recv_ranges.is_disjoint(&ranges) {
                        return Err(SubstringsProofError::DuplicateData(direction, ranges));
                    }
                    recv_ranges = recv_ranges.union(&ranges);
                }
            }

            // Make sure the ranges are within the bounds of the transcript
            let max = ranges
                .max()
                .ok_or(SubstringsProofError::InvalidOpening(id))?;
            let transcript_len = match direction {
                Direction::Sent => header.sent_len(),
                Direction::Received => header.recv_len(),
            };

            if max > transcript_len {
                return Err(SubstringsProofError::RangeOutOfBounds(id, max));
            }

            // Generate the expected encodings for the purported data in the opening.
            let encodings = get_value_ids(&ranges, direction)
                .map(|id| {
                    header
                        .encoder()
                        .encode_by_type(EncodingId::new(&id).to_inner(), &ValueType::U8)
                })
                .collect::<Vec<_>>();

            encoding_list.insert(id, encodings);

            // // Compute the expected hash of the commitment to make sure it is
            // // present in the merkle tree.
            // indices.push(id.to_inner() as usize);
            // expected_hashes.push(opening.recover(&encodings).hash());

            // // Make sure the length of data from the opening matches the commitment.
            let mut data = opening.into_data();
            if data.len() != ranges.len() {
                return Err(SubstringsProofError::InvalidOpening(id));
            }

            let dest = match direction {
                Direction::Sent => &mut sent,
                Direction::Received => &mut recv,
            };

            // Iterate over the ranges backwards, copying the data from the opening
            // then truncating it.
            for range in ranges.iter_ranges().rev() {
                let start = data.len() - range.len();
                dest[range].copy_from_slice(&data[start..]);
                data.truncate(start);
            }
        }

        Ok(encoding_list)
    }
    /// Verifies this proof and, if successful, returns the redacted sent and received transcripts.
    ///
    /// # Arguments
    ///
    /// * `header` - The session header.
    pub fn verify(
        self,
        header: &SessionHeader,
    ) -> Result<(RedactedTranscript, RedactedTranscript), SubstringsProofError> {
        let Self {
            openings,
            inclusion_proof,
        } = self;

        let mut indices: Vec<usize> = Vec::with_capacity(openings.len());
        let mut expected_hashes: Vec<Hash> = Vec::with_capacity(openings.len());
        let mut sent = vec![0u8; header.sent_len()];
        let mut recv = vec![0u8; header.recv_len()];
        let mut sent_ranges = RangeSet::default();
        let mut recv_ranges = RangeSet::default();
        let mut total_opened = 0u128;

        for (id, (info, opening)) in openings {
            let CommitmentInfo {
                ranges, direction, ..
            } = info;

            let opened_len = ranges.len();

            // Make sure the amount of data being proved is bounded.
            total_opened += opened_len as u128;
            if total_opened > MAX_TOTAL_COMMITTED_DATA as u128 {
                return Err(SubstringsProofError::MaxDataExceeded(total_opened as usize));
            }

            // Make sure the opening length matches the ranges length.
            if opening.data().len() != opened_len {
                return Err(SubstringsProofError::InvalidOpening(id));
            }

            // Make sure duplicate data is not opened.
            match direction {
                Direction::Sent => {
                    if !sent_ranges.is_disjoint(&ranges) {
                        return Err(SubstringsProofError::DuplicateData(direction, ranges));
                    }
                    sent_ranges = sent_ranges.union(&ranges);
                }
                Direction::Received => {
                    if !recv_ranges.is_disjoint(&ranges) {
                        return Err(SubstringsProofError::DuplicateData(direction, ranges));
                    }
                    recv_ranges = recv_ranges.union(&ranges);
                }
            }

            // Make sure the ranges are within the bounds of the transcript
            let max = ranges
                .max()
                .ok_or(SubstringsProofError::InvalidOpening(id))?;
            let transcript_len = match direction {
                Direction::Sent => header.sent_len(),
                Direction::Received => header.recv_len(),
            };

            if max > transcript_len {
                return Err(SubstringsProofError::RangeOutOfBounds(id, max));
            }

            // Generate the expected encodings for the purported data in the opening.
            let encodings = get_value_ids(&ranges, direction)
                .map(|id| {
                    header
                        .encoder()
                        .encode_by_type(EncodingId::new(&id).to_inner(), &ValueType::U8)
                })
                .collect::<Vec<_>>();

            // // Compute the expected hash of the commitment to make sure it is
            // // present in the merkle tree.
            indices.push(id.to_inner() as usize);
            expected_hashes.push(opening.recover(&encodings).hash());

            // // Make sure the length of data from the opening matches the commitment.
            let mut data = opening.into_data();
            if data.len() != ranges.len() {
                return Err(SubstringsProofError::InvalidOpening(id));
            }

            let dest = match direction {
                Direction::Sent => &mut sent,
                Direction::Received => &mut recv,
            };

            // Iterate over the ranges backwards, copying the data from the opening
            // then truncating it.
            for range in ranges.iter_ranges().rev() {
                let start = data.len() - range.len();
                dest[range].copy_from_slice(&data[start..]);
                data.truncate(start);
            }
        }

        // Verify that the expected hashes are present in the merkle tree.
        //
        // This proves the Prover committed to the purported data prior to the encoder
        // seed being revealed.
        inclusion_proof
            .verify(header.merkle_root(), &indices, &expected_hashes)
            .map_err(|e| SubstringsProofError::InvalidInclusionProof(e.to_string()))?;

        // Iterate over the unioned ranges and create TranscriptSlices for each.
        // This ensures that the slices are sorted and disjoint.
        let sent_slices = sent_ranges
            .iter_ranges()
            .map(|range| TranscriptSlice::new(range.clone(), sent[range].to_vec()))
            .collect();
        let recv_slices = recv_ranges
            .iter_ranges()
            .map(|range| TranscriptSlice::new(range.clone(), recv[range].to_vec()))
            .collect();

        Ok((
            RedactedTranscript::new(header.sent_len(), sent_slices),
            RedactedTranscript::new(header.recv_len(), recv_slices),
        ))
    }
}

#[cfg(test)]
mod tests {
    use crate::proof::{SessionHeader, SubstringsProof};

    #[test]
    fn test_verify_proof() {
        let session_header = r#"{"encoder_seed":[165,237,106,162,223,188,92,81,186,146,18,34,141,232,187,119,131,120,235,161,32,226,66,74,5,249,98,104,32,250,177,196],"merkle_root":[177,218,121,139,158,230,243,185,35,230,106,230,4,67,153,225,194,72,87,49,245,78,211,219,87,134,170,222,33,75,247,105],"sent_len":211,"recv_len":1617}"#;
        let substrings = r#"{"openings":{"0":[{"kind":"Blake3","ranges":[{"start":0,"end":211}],"direction":"Sent"},{"Blake3":{"data":[71,69,84,32,47,32,72,84,84,80,47,49,46,49,13,10,104,111,115,116,58,32,101,120,97,109,112,108,101,46,99,111,109,13,10,97,99,99,101,112,116,58,32,42,47,42,13,10,97,99,99,101,112,116,45,101,110,99,111,100,105,110,103,58,32,105,100,101,110,116,105,116,121,13,10,99,111,110,110,101,99,116,105,111,110,58,32,99,108,111,115,101,13,10,117,115,101,114,45,97,103,101,110,116,58,32,77,111,122,105,108,108,97,47,53,46,48,32,40,88,49,49,59,32,76,105,110,117,120,32,120,56,54,95,54,52,41,32,65,112,112,108,101,87,101,98,75,105,116,47,53,51,55,46,51,54,32,40,75,72,84,77,76,44,32,108,105,107,101,32,71,101,99,107,111,41,32,67,104,114,111,109,101,47,49,49,52,46,48,46,48,46,48,32,83,97,102,97,114,105,47,53,51,55,46,51,54,13,10,13,10],"nonce":[70,40,126,195,93,205,14,137,73,250,101,163,220,161,176,177,107,234,75,62,24,0,41,78,175,58,146,107,98,171,92,147]}}],"1":[{"kind":"Blake3","ranges":[{"start":0,"end":1617}],"direction":"Received"},{"Blake3":{"data":[72,84,84,80,47,49,46,49,32,50,48,48,32,79,75,13,10,65,103,101,58,32,51,51,50,52,53,49,13,10,67,97,99,104,101,45,67,111,110,116,114,111,108,58,32,109,97,120,45,97,103,101,61,54,48,52,56,48,48,13,10,67,111,110,116,101,110,116,45,84,121,112,101,58,32,116,101,120,116,47,104,116,109,108,59,32,99,104,97,114,115,101,116,61,85,84,70,45,56,13,10,68,97,116,101,58,32,77,111,110,44,32,50,50,32,65,112,114,32,50,48,50,52,32,49,53,58,48,50,58,49,48,32,71,77,84,13,10,69,116,97,103,58,32,34,51,49,52,55,53,50,54,57,52,55,43,103,122,105,112,43,105,100,101,110,116,34,13,10,69,120,112,105,114,101,115,58,32,77,111,110,44,32,50,57,32,65,112,114,32,50,48,50,52,32,49,53,58,48,50,58,49,48,32,71,77,84,13,10,76,97,115,116,45,77,111,100,105,102,105,101,100,58,32,84,104,117,44,32,49,55,32,79,99,116,32,50,48,49,57,32,48,55,58,49,56,58,50,54,32,71,77,84,13,10,83,101,114,118,101,114,58,32,69,67,65,99,99,32,40,100,99,100,47,55,68,50,69,41,13,10,86,97,114,121,58,32,65,99,99,101,112,116,45,69,110,99,111,100,105,110,103,13,10,88,45,67,97,99,104,101,58,32,72,73,84,13,10,67,111,110,116,101,110,116,45,76,101,110,103,116,104,58,32,49,50,53,54,13,10,67,111,110,110,101,99,116,105,111,110,58,32,99,108,111,115,101,13,10,13,10,60,33,100,111,99,116,121,112,101,32,104,116,109,108,62,10,60,104,116,109,108,62,10,60,104,101,97,100,62,10,32,32,32,32,60,116,105,116,108,101,62,69,120,97,109,112,108,101,32,68,111,109,97,105,110,60,47,116,105,116,108,101,62,10,10,32,32,32,32,60,109,101,116,97,32,99,104,97,114,115,101,116,61,34,117,116,102,45,56,34,32,47,62,10,32,32,32,32,60,109,101,116,97,32,104,116,116,112,45,101,113,117,105,118,61,34,67,111,110,116,101,110,116,45,116,121,112,101,34,32,99,111,110,116,101,110,116,61,34,116,101,120,116,47,104,116,109,108,59,32,99,104,97,114,115,101,116,61,117,116,102,45,56,34,32,47,62,10,32,32,32,32,60,109,101,116,97,32,110,97,109,101,61,34,118,105,101,119,112,111,114,116,34,32,99,111,110,116,101,110,116,61,34,119,105,100,116,104,61,100,101,118,105,99,101,45,119,105,100,116,104,44,32,105,110,105,116,105,97,108,45,115,99,97,108,101,61,49,34,32,47,62,10,32,32,32,32,60,115,116,121,108,101,32,116,121,112,101,61,34,116,101,120,116,47,99,115,115,34,62,10,32,32,32,32,98,111,100,121,32,123,10,32,32,32,32,32,32,32,32,98,97,99,107,103,114,111,117,110,100,45,99,111,108,111,114,58,32,35,102,48,102,48,102,50,59,10,32,32,32,32,32,32,32,32,109,97,114,103,105,110,58,32,48,59,10,32,32,32,32,32,32,32,32,112,97,100,100,105,110,103,58,32,48,59,10,32,32,32,32,32,32,32,32,102,111,110,116,45,102,97,109,105,108,121,58,32,45,97,112,112,108,101,45,115,121,115,116,101,109,44,32,115,121,115,116,101,109,45,117,105,44,32,66,108,105,110,107,77,97,99,83,121,115,116,101,109,70,111,110,116,44,32,34,83,101,103,111,101,32,85,73,34,44,32,34,79,112,101,110,32,83,97,110,115,34,44,32,34,72,101,108,118,101,116,105,99,97,32,78,101,117,101,34,44,32,72,101,108,118,101,116,105,99,97,44,32,65,114,105,97,108,44,32,115,97,110,115,45,115,101,114,105,102,59,10,32,32,32,32,32,32,32,32,10,32,32,32,32,125,10,32,32,32,32,100,105,118,32,123,10,32,32,32,32,32,32,32,32,119,105,100,116,104,58,32,54,48,48,112,120,59,10,32,32,32,32,32,32,32,32,109,97,114,103,105,110,58,32,53,101,109,32,97,117,116,111,59,10,32,32,32,32,32,32,32,32,112,97,100,100,105,110,103,58,32,50,101,109,59,10,32,32,32,32,32,32,32,32,98,97,99,107,103,114,111,117,110,100,45,99,111,108,111,114,58,32,35,102,100,102,100,102,102,59,10,32,32,32,32,32,32,32,32,98,111,114,100,101,114,45,114,97,100,105,117,115,58,32,48,46,53,101,109,59,10,32,32,32,32,32,32,32,32,98,111,120,45,115,104,97,100,111,119,58,32,50,112,120,32,51,112,120,32,55,112,120,32,50,112,120,32,114,103,98,97,40,48,44,48,44,48,44,48,46,48,50,41,59,10,32,32,32,32,125,10,32,32,32,32,97,58,108,105,110,107,44,32,97,58,118,105,115,105,116,101,100,32,123,10,32,32,32,32,32,32,32,32,99,111,108,111,114,58,32,35,51,56,52,56,56,102,59,10,32,32,32,32,32,32,32,32,116,101,120,116,45,100,101,99,111,114,97,116,105,111,110,58,32,110,111,110,101,59,10,32,32,32,32,125,10,32,32,32,32,64,109,101,100,105,97,32,40,109,97,120,45,119,105,100,116,104,58,32,55,48,48,112,120,41,32,123,10,32,32,32,32,32,32,32,32,100,105,118,32,123,10,32,32,32,32,32,32,32,32,32,32,32,32,109,97,114,103,105,110,58,32,48,32,97,117,116,111,59,10,32,32,32,32,32,32,32,32,32,32,32,32,119,105,100,116,104,58,32,97,117,116,111,59,10,32,32,32,32,32,32,32,32,125,10,32,32,32,32,125,10,32,32,32,32,60,47,115,116,121,108,101,62,32,32,32,32,10,60,47,104,101,97,100,62,10,10,60,98,111,100,121,62,10,60,100,105,118,62,10,32,32,32,32,60,104,49,62,69,120,97,109,112,108,101,32,68,111,109,97,105,110,60,47,104,49,62,10,32,32,32,32,60,112,62,84,104,105,115,32,100,111,109,97,105,110,32,105,115,32,102,111,114,32,117,115,101,32,105,110,32,105,108,108,117,115,116,114,97,116,105,118,101,32,101,120,97,109,112,108,101,115,32,105,110,32,100,111,99,117,109,101,110,116,115,46,32,89,111,117,32,109,97,121,32,117,115,101,32,116,104,105,115,10,32,32,32,32,100,111,109,97,105,110,32,105,110,32,108,105,116,101,114,97,116,117,114,101,32,119,105,116,104,111,117,116,32,112,114,105,111,114,32,99,111,111,114,100,105,110,97,116,105,111,110,32,111,114,32,97,115,107,105,110,103,32,102,111,114,32,112,101,114,109,105,115,115,105,111,110,46,60,47,112,62,10,32,32,32,32,60,112,62,60,97,32,104,114,101,102,61,34,104,116,116,112,115,58,47,47,119,119,119,46,105,97,110,97,46,111,114,103,47,100,111,109,97,105,110,115,47,101,120,97,109,112,108,101,34,62,77,111,114,101,32,105,110,102,111,114,109,97,116,105,111,110,46,46,46,60,47,97,62,60,47,112,62,10,60,47,100,105,118,62,10,60,47,98,111,100,121,62,10,60,47,104,116,109,108,62,10],"nonce":[243,119,180,179,116,75,195,225,175,33,9,58,138,69,4,115,174,123,97,163,233,177,176,172,75,101,109,103,179,137,246,45]}}]},"inclusion_proof":{"proof":[],"total_leaves":2}}"#;

        let substrings: SubstringsProof = serde_json::from_str(&substrings).unwrap();
        let session_header: SessionHeader =
            serde_json::from_str(&session_header).expect("Deserialization failed");

        let (sent, recv) = substrings.verify(&session_header).unwrap();

        let req = String::from_utf8(recv.data().to_vec()).unwrap();

        let res = String::from_utf8(sent.data().to_vec()).unwrap();

        println!("{}", req);
        println!("{}", res);
        assert!(req.starts_with("HTTP"))
    }
}
