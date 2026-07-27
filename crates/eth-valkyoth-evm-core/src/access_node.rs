use crate::{
    EVM_MAX_WARM_ADDRESSES, EVM_MAX_WARM_STORAGE_SLOTS, EvmAccessAttempt, EvmAccessProfile,
    EvmAccessStatus, EvmAccessTracker, EvmAddress, EvmCoreError, EvmWord,
    access_radix::{AccessKey, RadixSet, SetCheckpoint},
};

const ADDRESS_BITS: usize = EvmAddress::LEN * 8;
const STORAGE_KEY_BITS: usize = ADDRESS_BITS + EvmWord::LEN * 8;
const BYTE_BIT_MASKS: [u8; 8] = [0x80, 0x40, 0x20, 0x10, 0x08, 0x04, 0x02, 0x01];

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct StorageKey {
    address: EvmAddress,
    key: EvmWord,
}

fn byte_bit(bytes: &[u8], index: usize) -> bool {
    let byte = bytes.get(index / 8).copied().unwrap_or_default();
    let mask = BYTE_BIT_MASKS.get(index % 8).copied().unwrap_or_default();
    byte & mask != 0
}

impl AccessKey for EvmAddress {
    const BITS: usize = ADDRESS_BITS;

    fn bit(&self, index: usize) -> bool {
        byte_bit(self.as_bytes(), index)
    }

    fn wipe(&mut self) {
        EvmAddress::wipe(self);
    }
}

impl AccessKey for StorageKey {
    const BITS: usize = STORAGE_KEY_BITS;

    fn bit(&self, index: usize) -> bool {
        if index < ADDRESS_BITS {
            return byte_bit(self.address.as_bytes(), index);
        }
        byte_bit(self.key.as_be_bytes(), index.saturating_sub(ADDRESS_BITS))
    }

    fn wipe(&mut self) {
        EvmAddress::wipe(&mut self.address);
        EvmWord::wipe(&mut self.key);
    }
}

/// Opaque LIFO checkpoint for the node-scale access tracker.
#[derive(Debug, Eq, PartialEq)]
pub struct EvmNodeAccessCheckpoint {
    addresses: SetCheckpoint,
    storage: SetCheckpoint,
    generation: u64,
    depth: usize,
}

/// Pre-reserved compressed-radix tracker for node-scale execution.
///
/// Construction performs all allocation. Lookup and insertion are `O(w)`,
/// where `w` is the fixed address or address/storage key width. Reverting a
/// checkpoint is `O(k)` for the `k` unique insertions made after it and never
/// rebuilds retained outer-scope state.
#[derive(Debug, Eq, PartialEq)]
pub struct EvmNodeAccessTracker {
    addresses: RadixSet<EvmAddress>,
    storage: RadixSet<StorageKey>,
    attempt: Option<EvmNodeAccessCheckpoint>,
    generation: u64,
    checkpoint_depth: usize,
}

impl EvmNodeAccessTracker {
    /// Creates an empty tracker and reserves every node and undo entry.
    pub fn try_new(address_capacity: usize, storage_capacity: usize) -> Result<Self, EvmCoreError> {
        Ok(Self {
            addresses: RadixSet::try_new(address_capacity, EVM_MAX_WARM_ADDRESSES)?,
            storage: RadixSet::try_new(storage_capacity, EVM_MAX_WARM_STORAGE_SLOTS)?,
            attempt: None,
            generation: 0,
            checkpoint_depth: 0,
        })
    }

    /// Returns the number of warmed addresses.
    #[must_use]
    pub fn address_len(&self) -> usize {
        self.addresses.len()
    }

    /// Returns the number of warmed storage slots.
    #[must_use]
    pub fn storage_len(&self) -> usize {
        self.storage.len()
    }

    /// Returns retained node-plus-undo capacities for both indexes.
    #[must_use]
    pub fn allocation_capacities(&self) -> (usize, usize) {
        (
            self.addresses.allocation_capacity(),
            self.storage.allocation_capacity(),
        )
    }

    /// Returns actual maximum lookup depths for complexity evidence.
    pub fn max_lookup_depths(&self) -> Result<(usize, usize), EvmCoreError> {
        Ok((self.addresses.max_depth()?, self.storage.max_depth()?))
    }

    /// Destructively clears all transaction warmth while retaining allocation.
    pub fn reset_transaction(&mut self) -> Result<(), EvmCoreError> {
        <Self as EvmAccessTracker>::reset_transaction(self)
    }

    /// Marks an address warm.
    pub fn warm_address(&mut self, address: EvmAddress) -> Result<EvmAccessStatus, EvmCoreError> {
        <Self as EvmAccessTracker>::warm_address(self, address)
    }

    /// Marks an address/storage pair warm.
    pub fn warm_storage(
        &mut self,
        address: EvmAddress,
        key: EvmWord,
    ) -> Result<EvmAccessStatus, EvmCoreError> {
        <Self as EvmAccessTracker>::warm_storage(self, address, key)
    }

    fn validate_checkpoint(
        &self,
        checkpoint: &EvmNodeAccessCheckpoint,
    ) -> Result<(), EvmCoreError> {
        if checkpoint.generation != self.generation
            || checkpoint.depth == 0
            || checkpoint.depth != self.checkpoint_depth
        {
            return Err(EvmCoreError::StateAccessAttemptMissing);
        }
        Ok(())
    }
}

impl EvmAccessTracker for EvmNodeAccessTracker {
    type Checkpoint = EvmNodeAccessCheckpoint;

    fn profile(&self) -> EvmAccessProfile {
        EvmAccessProfile::NodeFixedWidthRadix
    }

    fn address_len(&self) -> usize {
        self.addresses.len()
    }

    fn storage_len(&self) -> usize {
        self.storage.len()
    }

    fn reset_transaction(&mut self) -> Result<(), EvmCoreError> {
        let generation = self
            .generation
            .checked_add(1)
            .ok_or(EvmCoreError::StateAccessTrackerCorrupt)?;
        self.addresses.clear();
        self.storage.clear();
        self.attempt = None;
        self.checkpoint_depth = 0;
        self.generation = generation;
        Ok(())
    }

    fn checkpoint(&mut self) -> Result<Self::Checkpoint, EvmCoreError> {
        let depth = self
            .checkpoint_depth
            .checked_add(1)
            .ok_or(EvmCoreError::StateAccessTrackerCorrupt)?;
        let checkpoint = EvmNodeAccessCheckpoint {
            addresses: self.addresses.checkpoint(),
            storage: self.storage.checkpoint(),
            generation: self.generation,
            depth,
        };
        self.checkpoint_depth = depth;
        Ok(checkpoint)
    }

    fn commit(&mut self, checkpoint: Self::Checkpoint) -> Result<(), EvmCoreError> {
        self.validate_checkpoint(&checkpoint)?;
        self.checkpoint_depth = self.checkpoint_depth.saturating_sub(1);
        if self.checkpoint_depth == 0 {
            self.addresses.clear_undo();
            self.storage.clear_undo();
        }
        Ok(())
    }

    fn revert(&mut self, checkpoint: Self::Checkpoint) -> Result<(), EvmCoreError> {
        self.validate_checkpoint(&checkpoint)?;
        self.addresses.restore(checkpoint.addresses)?;
        self.storage.restore(checkpoint.storage)?;
        self.checkpoint_depth = self.checkpoint_depth.saturating_sub(1);
        if self.checkpoint_depth == 0 {
            self.addresses.clear_undo();
            self.storage.clear_undo();
        }
        Ok(())
    }

    fn begin_attempt(&mut self) -> Result<(), EvmCoreError> {
        if self.attempt.is_some() {
            return Err(EvmCoreError::StateAccessAttemptAlreadyActive);
        }
        self.attempt = Some(self.checkpoint()?);
        Ok(())
    }

    fn finish_attempt(&mut self, outcome: EvmAccessAttempt) -> Result<(), EvmCoreError> {
        let checkpoint = self
            .attempt
            .take()
            .ok_or(EvmCoreError::StateAccessAttemptMissing)?;
        match outcome {
            EvmAccessAttempt::Commit => self.commit(checkpoint),
            EvmAccessAttempt::Rollback => self.revert(checkpoint),
        }
    }

    fn warm_address(&mut self, address: EvmAddress) -> Result<EvmAccessStatus, EvmCoreError> {
        if !self.addresses.can_insert(address)? {
            return Ok(EvmAccessStatus::Warm);
        }
        self.addresses.insert_known_absent(address)?;
        Ok(EvmAccessStatus::Cold)
    }

    fn warm_storage(
        &mut self,
        address: EvmAddress,
        key: EvmWord,
    ) -> Result<EvmAccessStatus, EvmCoreError> {
        let storage = StorageKey { address, key };
        if self.storage.contains(storage)? {
            return Ok(EvmAccessStatus::Warm);
        }
        let insert_address = self.addresses.can_insert(address)?;
        let _ = self.storage.can_insert(storage)?;
        if insert_address {
            self.addresses.insert_known_absent(address)?;
        }
        self.storage.insert_known_absent(storage)?;
        Ok(EvmAccessStatus::Cold)
    }
}
