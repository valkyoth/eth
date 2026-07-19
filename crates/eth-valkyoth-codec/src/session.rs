use crate::{DecodeError, DecodeLimits};

mod policy;
pub use policy::DecodeSessionPolicy;

/// Non-copyable work capability for one untrusted decode operation.
///
/// The type intentionally provides no clone or reset operation. Nested
/// consumers receive `&mut DecodeSession` and therefore share one conserved
/// set of counters.
#[derive(Debug, Eq, PartialEq)]
pub struct DecodeSession {
    policy: DecodeSessionPolicy,
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
    /// Starts a fresh session after validating the policy relationships.
    pub const fn new(policy: DecodeSessionPolicy) -> Result<Self, DecodeError> {
        match policy.validate_relationships() {
            Ok(()) => {}
            Err(error) => return Err(error),
        }
        Ok(Self {
            policy,
            encoded_bytes: 0,
            rlp_headers: 0,
            items: 0,
            max_nesting_depth: 0,
            allocation_capacity: 0,
            proof_nodes: 0,
            hashes: 0,
            hash_bytes: 0,
            nibbles: 0,
            value_bytes: 0,
            total_work: 0,
        })
    }

    /// Returns the immutable policy used by this session.
    #[must_use]
    pub const fn policy(&self) -> DecodeSessionPolicy {
        self.policy
    }

    /// Returns the structural limits used by this session.
    #[must_use]
    pub const fn limits(&self) -> DecodeLimits {
        self.policy.limits()
    }

    /// Returns cumulative encoded bytes charged as scanned.
    #[must_use]
    pub const fn encoded_bytes(&self) -> usize {
        self.encoded_bytes
    }

    /// Returns cumulative RLP headers charged as visited.
    #[must_use]
    pub const fn rlp_headers(&self) -> usize {
        self.rlp_headers
    }

    /// Returns cumulative decoded items charged.
    #[must_use]
    pub const fn items(&self) -> usize {
        self.items
    }

    /// Returns the deepest nesting level observed.
    #[must_use]
    pub const fn max_nesting_depth(&self) -> usize {
        self.max_nesting_depth
    }

    /// Returns cumulative requested allocation capacity.
    #[must_use]
    pub const fn allocation_capacity(&self) -> usize {
        self.allocation_capacity
    }

    /// Returns cumulative proof nodes charged.
    #[must_use]
    pub const fn proof_nodes(&self) -> usize {
        self.proof_nodes
    }

    /// Returns cumulative hash operations charged.
    #[must_use]
    pub const fn hashes(&self) -> usize {
        self.hashes
    }

    /// Returns cumulative bytes charged to hash operations.
    #[must_use]
    pub const fn hash_bytes(&self) -> usize {
        self.hash_bytes
    }

    /// Returns cumulative compact-path nibbles charged.
    #[must_use]
    pub const fn nibbles(&self) -> usize {
        self.nibbles
    }

    /// Returns cumulative trie-value bytes charged.
    #[must_use]
    pub const fn value_bytes(&self) -> usize {
        self.value_bytes
    }

    /// Returns aggregate charged work units.
    #[must_use]
    pub const fn total_work(&self) -> usize {
        self.total_work
    }

    /// Checks an outer input length without resetting or charging the session.
    pub fn check_input_len(&self, len: usize) -> Result<(), DecodeError> {
        self.policy.limits().check_input_len(len)
    }

    /// Checks a per-list item count.
    pub fn check_list_count(&self, count: usize) -> Result<(), DecodeError> {
        self.policy.limits().check_list_count(count)
    }

    /// Checks and records a nesting depth.
    pub fn check_nesting_depth(&mut self, depth: usize) -> Result<(), DecodeError> {
        self.policy.limits().check_nesting_depth(depth)?;
        if depth > self.max_nesting_depth {
            let additional_depth = depth
                .checked_sub(self.max_nesting_depth)
                .ok_or(DecodeError::WorkExceeded)?;
            let work = self.checked_work(additional_depth)?;
            self.max_nesting_depth = depth;
            self.total_work = work;
        }
        Ok(())
    }

    /// Charges encoded bytes scanned by a parser pass.
    pub fn account_encoded_bytes(&mut self, count: usize) -> Result<(), DecodeError> {
        let encoded = checked_counter(
            self.encoded_bytes,
            count,
            self.policy.max_encoded_bytes(),
            DecodeError::EncodedBytesExceeded,
        )?;
        let work = self.checked_work(count)?;
        self.encoded_bytes = encoded;
        self.total_work = work;
        Ok(())
    }

    /// Charges RLP headers visited by a parser pass.
    pub fn account_rlp_headers(&mut self, count: usize) -> Result<(), DecodeError> {
        let headers = checked_counter(
            self.rlp_headers,
            count,
            self.policy.max_rlp_headers(),
            DecodeError::RlpHeaderCountExceeded,
        )?;
        let work = self.checked_work(count)?;
        self.rlp_headers = headers;
        self.total_work = work;
        Ok(())
    }

    /// Atomically charges one RLP reparse across all affected counters.
    pub fn account_rlp_reparse(
        &mut self,
        encoded_bytes: usize,
        headers: usize,
        items: usize,
    ) -> Result<(), DecodeError> {
        let encoded = checked_counter(
            self.encoded_bytes,
            encoded_bytes,
            self.policy.max_encoded_bytes(),
            DecodeError::EncodedBytesExceeded,
        )?;
        let rlp_headers = checked_counter(
            self.rlp_headers,
            headers,
            self.policy.max_rlp_headers(),
            DecodeError::RlpHeaderCountExceeded,
        )?;
        let item_count = checked_counter(
            self.items,
            items,
            self.policy.limits().max_total_items,
            DecodeError::ItemCountExceeded,
        )?;
        let work_charge = encoded_bytes
            .checked_add(headers)
            .and_then(|work| work.checked_add(items))
            .ok_or(DecodeError::WorkExceeded)?;
        let work = self.checked_work(work_charge)?;
        self.encoded_bytes = encoded;
        self.rlp_headers = rlp_headers;
        self.items = item_count;
        self.total_work = work;
        Ok(())
    }

    /// Charges decoded items.
    pub fn account_items(&mut self, count: usize) -> Result<(), DecodeError> {
        let items = checked_counter(
            self.items,
            count,
            self.policy.limits().max_total_items,
            DecodeError::ItemCountExceeded,
        )?;
        let work = self.checked_work(count)?;
        self.items = items;
        self.total_work = work;
        Ok(())
    }

    /// Charges requested allocation capacity before allocation occurs.
    pub fn account_allocation_capacity(&mut self, capacity: usize) -> Result<(), DecodeError> {
        let allocation = checked_counter(
            self.allocation_capacity,
            capacity,
            self.policy.limits().max_total_allocation,
            DecodeError::AllocationExceeded,
        )?;
        let work = self.checked_work(capacity)?;
        self.allocation_capacity = allocation;
        self.total_work = work;
        Ok(())
    }

    /// Charges proof nodes visited.
    pub fn account_proof_nodes(&mut self, count: usize) -> Result<(), DecodeError> {
        let nodes = checked_counter(
            self.proof_nodes,
            count,
            self.policy.limits().max_proof_nodes,
            DecodeError::ProofTooLarge,
        )?;
        let work = self.checked_work(count)?;
        self.proof_nodes = nodes;
        self.total_work = work;
        Ok(())
    }

    /// Charges one or more hashes and their complete input bytes atomically.
    pub fn account_hashes(&mut self, count: usize, bytes: usize) -> Result<(), DecodeError> {
        let hashes = checked_counter(
            self.hashes,
            count,
            self.policy.max_hashes(),
            DecodeError::HashCountExceeded,
        )?;
        let hash_bytes = checked_counter(
            self.hash_bytes,
            bytes,
            self.policy.max_hash_bytes(),
            DecodeError::HashBytesExceeded,
        )?;
        let work_delta = count.checked_add(bytes).ok_or(DecodeError::WorkExceeded)?;
        let work = self.checked_work(work_delta)?;
        self.hashes = hashes;
        self.hash_bytes = hash_bytes;
        self.total_work = work;
        Ok(())
    }

    /// Checks whether a future set of hashes fits without committing counters.
    ///
    /// Callers must still use [`Self::account_hashes`] immediately before each
    /// admitted hash operation. This preflight prevents a proof from starting
    /// cryptographic work when its complete hash shape is already over budget.
    pub fn check_hash_capacity(&self, count: usize, bytes: usize) -> Result<(), DecodeError> {
        let _ = checked_counter(
            self.hashes,
            count,
            self.policy.max_hashes(),
            DecodeError::HashCountExceeded,
        )?;
        let _ = checked_counter(
            self.hash_bytes,
            bytes,
            self.policy.max_hash_bytes(),
            DecodeError::HashBytesExceeded,
        )?;
        let work_delta = count.checked_add(bytes).ok_or(DecodeError::WorkExceeded)?;
        let _ = self.checked_work(work_delta)?;
        Ok(())
    }

    /// Charges compact-path nibbles inspected by a trie operation.
    pub fn account_nibbles(&mut self, count: usize) -> Result<(), DecodeError> {
        let nibbles = checked_counter(
            self.nibbles,
            count,
            self.policy.max_nibbles(),
            DecodeError::NibbleCountExceeded,
        )?;
        let work = self.checked_work(count)?;
        self.nibbles = nibbles;
        self.total_work = work;
        Ok(())
    }

    /// Charges trie value bytes exposed or compared by an operation.
    pub fn account_value_bytes(&mut self, count: usize) -> Result<(), DecodeError> {
        let value_bytes = checked_counter(
            self.value_bytes,
            count,
            self.policy.max_value_bytes(),
            DecodeError::ValueBytesExceeded,
        )?;
        let work = self.checked_work(count)?;
        self.value_bytes = value_bytes;
        self.total_work = work;
        Ok(())
    }

    fn checked_work(&self, count: usize) -> Result<usize, DecodeError> {
        checked_counter(
            self.total_work,
            count,
            self.policy.max_total_work(),
            DecodeError::WorkExceeded,
        )
    }
}

fn checked_counter(
    current: usize,
    increment: usize,
    maximum: usize,
    error: DecodeError,
) -> Result<usize, DecodeError> {
    let next = current.checked_add(increment).ok_or(error)?;
    if next > maximum {
        return Err(error);
    }
    Ok(next)
}

#[cfg(test)]
#[path = "session_tests.rs"]
mod tests;
