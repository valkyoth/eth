use crate::DecodeError;

use super::{DecodeSession, checked_counter};

/// Opaque accounting snapshot for noncommitting capacity checks.
///
/// This value carries no authority to perform work and exposes no public
/// constructor or mutable fields.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DecodeSessionCharges {
    encoded_bytes: usize,
    rlp_headers: usize,
    items: usize,
    max_nesting_depth: usize,
    allocation_capacity: usize,
    proof_nodes: usize,
    hashes: usize,
    hash_bytes: usize,
    nibbles: usize,
    value_bytes: usize,
    total_work: usize,
}

impl DecodeSession {
    /// Returns an opaque snapshot of every charged work domain.
    #[must_use]
    pub const fn charges(&self) -> DecodeSessionCharges {
        DecodeSessionCharges {
            encoded_bytes: self.encoded_bytes,
            rlp_headers: self.rlp_headers,
            items: self.items,
            max_nesting_depth: self.max_nesting_depth,
            allocation_capacity: self.allocation_capacity,
            proof_nodes: self.proof_nodes,
            hashes: self.hashes,
            hash_bytes: self.hash_bytes,
            nibbles: self.nibbles,
            value_bytes: self.value_bytes,
            total_work: self.total_work,
        }
    }

    /// Runs work against this session and returns its repeatable charges.
    ///
    /// The observed nesting requirement is retained for transfer to another
    /// session. Applying it to this session charges only a deeper high-water
    /// mark, so already-established depth is not charged twice.
    pub fn measure_replay_charges(
        &mut self,
        operation: impl FnOnce(&mut Self),
    ) -> Result<DecodeSessionCharges, DecodeError> {
        let before = self.charges();
        operation(self);
        self.replay_charges_since(before)
    }

    fn replay_charges_since(
        &self,
        before: DecodeSessionCharges,
    ) -> Result<DecodeSessionCharges, DecodeError> {
        let depth_work = self
            .max_nesting_depth
            .checked_sub(before.max_nesting_depth)
            .ok_or(DecodeError::WorkExceeded)?;
        let additive_work = difference(self.total_work, before.total_work)?
            .checked_sub(depth_work)
            .ok_or(DecodeError::WorkExceeded)?;
        let replay_work = additive_work
            .checked_add(self.max_nesting_depth)
            .ok_or(DecodeError::WorkExceeded)?;
        Ok(DecodeSessionCharges {
            encoded_bytes: difference(self.encoded_bytes, before.encoded_bytes)?,
            rlp_headers: difference(self.rlp_headers, before.rlp_headers)?,
            items: difference(self.items, before.items)?,
            max_nesting_depth: self.max_nesting_depth,
            allocation_capacity: difference(self.allocation_capacity, before.allocation_capacity)?,
            proof_nodes: difference(self.proof_nodes, before.proof_nodes)?,
            hashes: difference(self.hashes, before.hashes)?,
            hash_bytes: difference(self.hash_bytes, before.hash_bytes)?,
            nibbles: difference(self.nibbles, before.nibbles)?,
            value_bytes: difference(self.value_bytes, before.value_bytes)?,
            total_work: replay_work,
        })
    }

    /// Checks whether every charge in `additional` fits without committing.
    pub fn check_remaining_capacity(
        &self,
        additional: DecodeSessionCharges,
    ) -> Result<(), DecodeError> {
        let _ = self.project_charges(additional)?;
        Ok(())
    }

    /// Atomically adds an opaque charge description to this session.
    pub fn account_charges(&mut self, additional: DecodeSessionCharges) -> Result<(), DecodeError> {
        let projected = self.project_charges(additional)?;
        self.encoded_bytes = projected.encoded_bytes;
        self.rlp_headers = projected.rlp_headers;
        self.items = projected.items;
        self.max_nesting_depth = projected.max_nesting_depth;
        self.allocation_capacity = projected.allocation_capacity;
        self.proof_nodes = projected.proof_nodes;
        self.hashes = projected.hashes;
        self.hash_bytes = projected.hash_bytes;
        self.nibbles = projected.nibbles;
        self.value_bytes = projected.value_bytes;
        self.total_work = projected.total_work;
        Ok(())
    }

    fn project_charges(
        &self,
        additional: DecodeSessionCharges,
    ) -> Result<DecodeSessionCharges, DecodeError> {
        let limits = self.policy.limits();
        limits.check_nesting_depth(additional.max_nesting_depth)?;
        let max_nesting_depth = self.max_nesting_depth.max(additional.max_nesting_depth);
        let depth_work = max_nesting_depth
            .checked_sub(self.max_nesting_depth)
            .ok_or(DecodeError::WorkExceeded)?;
        let additive_work = additional
            .total_work
            .checked_sub(additional.max_nesting_depth)
            .ok_or(DecodeError::WorkExceeded)?;
        let work_delta = additive_work
            .checked_add(depth_work)
            .ok_or(DecodeError::WorkExceeded)?;

        Ok(DecodeSessionCharges {
            encoded_bytes: checked_counter(
                self.encoded_bytes,
                additional.encoded_bytes,
                self.policy.max_encoded_bytes(),
                DecodeError::EncodedBytesExceeded,
            )?,
            rlp_headers: checked_counter(
                self.rlp_headers,
                additional.rlp_headers,
                self.policy.max_rlp_headers(),
                DecodeError::RlpHeaderCountExceeded,
            )?,
            items: checked_counter(
                self.items,
                additional.items,
                limits.max_total_items,
                DecodeError::ItemCountExceeded,
            )?,
            max_nesting_depth,
            allocation_capacity: checked_counter(
                self.allocation_capacity,
                additional.allocation_capacity,
                limits.max_total_allocation,
                DecodeError::AllocationExceeded,
            )?,
            proof_nodes: checked_counter(
                self.proof_nodes,
                additional.proof_nodes,
                limits.max_proof_nodes,
                DecodeError::ProofTooLarge,
            )?,
            hashes: checked_counter(
                self.hashes,
                additional.hashes,
                self.policy.max_hashes(),
                DecodeError::HashCountExceeded,
            )?,
            hash_bytes: checked_counter(
                self.hash_bytes,
                additional.hash_bytes,
                self.policy.max_hash_bytes(),
                DecodeError::HashBytesExceeded,
            )?,
            nibbles: checked_counter(
                self.nibbles,
                additional.nibbles,
                self.policy.max_nibbles(),
                DecodeError::NibbleCountExceeded,
            )?,
            value_bytes: checked_counter(
                self.value_bytes,
                additional.value_bytes,
                self.policy.max_value_bytes(),
                DecodeError::ValueBytesExceeded,
            )?,
            total_work: checked_counter(
                self.total_work,
                work_delta,
                self.policy.max_total_work(),
                DecodeError::WorkExceeded,
            )?,
        })
    }
}

fn difference(current: usize, before: usize) -> Result<usize, DecodeError> {
    current.checked_sub(before).ok_or(DecodeError::WorkExceeded)
}
