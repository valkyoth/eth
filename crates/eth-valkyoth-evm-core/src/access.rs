use crate::{EvmAddress, EvmCoreError, EvmWord};

const EIP2930_ADDRESS_GAS: u64 = 2_400;
const EIP2930_STORAGE_KEY_GAS: u64 = 1_900;
const TRANSACTION_BASE_GAS: u64 = 21_000;
const PRAGUE_INITIAL_WARM_ADDRESSES: u64 = 20;
const MAX_PAID_WARM_ADDRESSES_U64: u64 =
    (crate::EVM_MAX_GAS_LIMIT - TRANSACTION_BASE_GAS) / EIP2930_ADDRESS_GAS;
const MAX_WARM_ADDRESSES_U64: u64 = MAX_PAID_WARM_ADDRESSES_U64 + PRAGUE_INITIAL_WARM_ADDRESSES;
const MAX_WARM_STORAGE_SLOTS_U64: u64 = crate::EVM_MAX_GAS_LIMIT / EIP2930_STORAGE_KEY_GAS;

/// Maximum warmed addresses admitted by the bootstrap execution policy.
///
/// This covers the maximum paid EIP-2930 address entries after base
/// transaction gas plus sender, destination/create address, coinbase, and the
/// 17 precompiles active through Prague. Fork-specific policy may use less.
pub const EVM_MAX_WARM_ADDRESSES: usize = 416_677;

/// Maximum warmed storage slots admitted by the bootstrap execution policy.
///
/// The ceiling covers the maximum number of EIP-2930 storage-key entries that
/// can fit under [`crate::EVM_MAX_GAS_LIMIT`] at 1,900 intrinsic gas each.
pub const EVM_MAX_WARM_STORAGE_SLOTS: usize = 526_315;

const _: () = {
    assert!(MAX_PAID_WARM_ADDRESSES_U64 == 416_657);
    assert!(MAX_WARM_ADDRESSES_U64 == 416_677);
    assert!(MAX_WARM_STORAGE_SLOTS_U64 == 526_315);
    assert!(
        MAX_PAID_WARM_ADDRESSES_U64 * EIP2930_ADDRESS_GAS + TRANSACTION_BASE_GAS
            <= crate::EVM_MAX_GAS_LIMIT
    );
    assert!(MAX_WARM_STORAGE_SLOTS_U64 * EIP2930_STORAGE_KEY_GAS <= crate::EVM_MAX_GAS_LIMIT);
};

/// Warm/cold access classification for one execution attempt.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EvmAccessStatus {
    /// The account or storage slot was already warm.
    Warm,
    /// The account or storage slot was cold and was marked warm by this access.
    Cold,
}

/// Execution-attempt disposition.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EvmAccessAttempt {
    /// Preserve warmth added by the successful attempt.
    Commit,
    /// Restore the exact warmth present before the attempt.
    Rollback,
}

/// Opaque LIFO checkpoint for one access-tracker scope.
#[derive(Debug, Eq, PartialEq)]
pub struct EvmAccessCheckpoint {
    pub(crate) address_len: usize,
    pub(crate) storage_len: usize,
    pub(crate) generation: u64,
    pub(crate) depth: usize,
}

/// Complexity profile of an access-tracker implementation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EvmAccessProfile {
    /// Allocation-free arrays with linear membership checks.
    EmbeddedLinear,
    /// Pre-reserved compressed-radix indexes with fixed-width lookup bounds.
    NodeFixedWidthRadix,
}

/// Injectable transaction-global EIP-2929 warmth tracker.
///
/// Implementations must make `warm_storage` atomic: if either the address or
/// storage capacity is unavailable, neither entry may be added. Checkpoints
/// are nested and LIFO because an EIP-2929 scope revert restores warmth to the
/// exact state present before that scope was entered.
pub trait EvmAccessTracker {
    /// Non-forgeable implementation-defined checkpoint token.
    type Checkpoint;

    /// Returns the implementation's documented complexity profile.
    fn profile(&self) -> EvmAccessProfile;

    /// Returns the number of warmed addresses.
    fn address_len(&self) -> usize;

    /// Returns the number of warmed storage slots.
    fn storage_len(&self) -> usize;

    /// Destructively clears all transaction warmth and active attempt state.
    fn reset_transaction(&mut self) -> Result<(), EvmCoreError>;

    /// Starts one nested warmth checkpoint.
    fn checkpoint(&mut self) -> Result<Self::Checkpoint, EvmCoreError>;

    /// Commits the active warmth checkpoint.
    fn commit(&mut self, checkpoint: Self::Checkpoint) -> Result<(), EvmCoreError>;

    /// Reverts the active warmth checkpoint.
    fn revert(&mut self, checkpoint: Self::Checkpoint) -> Result<(), EvmCoreError>;

    /// Starts one root execution attempt.
    fn begin_attempt(&mut self) -> Result<(), EvmCoreError>;

    /// Commits or rolls back the active root attempt.
    fn finish_attempt(&mut self, outcome: EvmAccessAttempt) -> Result<(), EvmCoreError>;

    /// Marks an address warm and returns its previous status.
    fn warm_address(&mut self, address: EvmAddress) -> Result<EvmAccessStatus, EvmCoreError>;

    /// Marks an address/storage pair warm and returns its previous status.
    fn warm_storage(
        &mut self,
        address: EvmAddress,
        key: EvmWord,
    ) -> Result<EvmAccessStatus, EvmCoreError>;
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct StorageSlotAccess {
    address: EvmAddress,
    key: EvmWord,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct AccessCheckpoint {
    address_len: usize,
    storage_len: usize,
}

/// Allocation-free fixed-array tracker for explicit embedded profiles.
///
/// Membership costs at most `ADDRESSES` address comparisons or `STORAGE`
/// storage comparisons per operation. Node deployments should use
/// [`crate::EvmNodeAccessTracker`] instead.
#[derive(Debug, Eq, PartialEq)]
pub struct EvmEmbeddedAccessTracker<const ADDRESSES: usize, const STORAGE: usize> {
    addresses: [EvmAddress; ADDRESSES],
    address_len: usize,
    storage: [StorageSlotAccess; STORAGE],
    storage_len: usize,
    attempt: Option<EvmAccessCheckpoint>,
    generation: u64,
    checkpoint_depth: usize,
}

impl<const ADDRESSES: usize, const STORAGE: usize> EvmEmbeddedAccessTracker<ADDRESSES, STORAGE> {
    /// Creates an empty embedded tracker.
    pub const fn try_new() -> Result<Self, EvmCoreError> {
        if ADDRESSES == 0 || STORAGE == 0 {
            return Err(EvmCoreError::StateAccessListTooSmall);
        }
        if ADDRESSES > EVM_MAX_WARM_ADDRESSES || STORAGE > EVM_MAX_WARM_STORAGE_SLOTS {
            return Err(EvmCoreError::StateAccessCapacityTooLarge);
        }
        Ok(Self {
            addresses: [EvmAddress::ZERO; ADDRESSES],
            address_len: 0,
            storage: [StorageSlotAccess {
                address: EvmAddress::ZERO,
                key: EvmWord::ZERO,
            }; STORAGE],
            storage_len: 0,
            attempt: None,
            generation: 0,
            checkpoint_depth: 0,
        })
    }

    /// Returns the number of warmed addresses.
    #[must_use]
    pub const fn address_len(&self) -> usize {
        self.address_len
    }

    /// Returns the number of warmed storage slots.
    #[must_use]
    pub const fn storage_len(&self) -> usize {
        self.storage_len
    }

    /// Destructively clears all transaction warmth.
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

    fn restore(&mut self, checkpoint: AccessCheckpoint) -> Result<(), EvmCoreError> {
        if checkpoint.address_len > self.address_len || checkpoint.storage_len > self.storage_len {
            return Err(EvmCoreError::StateAccessTrackerCorrupt);
        }
        for address in self
            .addresses
            .iter_mut()
            .skip(checkpoint.address_len)
            .take(self.address_len.saturating_sub(checkpoint.address_len))
        {
            *address = EvmAddress::ZERO;
        }
        for slot in self
            .storage
            .iter_mut()
            .skip(checkpoint.storage_len)
            .take(self.storage_len.saturating_sub(checkpoint.storage_len))
        {
            *slot = StorageSlotAccess {
                address: EvmAddress::ZERO,
                key: EvmWord::ZERO,
            };
        }
        self.address_len = checkpoint.address_len;
        self.storage_len = checkpoint.storage_len;
        Ok(())
    }

    fn validate_checkpoint(&self, checkpoint: &EvmAccessCheckpoint) -> Result<(), EvmCoreError> {
        if checkpoint.generation != self.generation
            || checkpoint.depth == 0
            || checkpoint.depth != self.checkpoint_depth
        {
            return Err(EvmCoreError::StateAccessAttemptMissing);
        }
        Ok(())
    }
}

impl<const ADDRESSES: usize, const STORAGE: usize> EvmAccessTracker
    for EvmEmbeddedAccessTracker<ADDRESSES, STORAGE>
{
    type Checkpoint = EvmAccessCheckpoint;

    fn profile(&self) -> EvmAccessProfile {
        EvmAccessProfile::EmbeddedLinear
    }

    fn address_len(&self) -> usize {
        self.address_len
    }

    fn storage_len(&self) -> usize {
        self.storage_len
    }

    fn reset_transaction(&mut self) -> Result<(), EvmCoreError> {
        let generation = self
            .generation
            .checked_add(1)
            .ok_or(EvmCoreError::StateAccessTrackerCorrupt)?;
        self.restore(AccessCheckpoint {
            address_len: 0,
            storage_len: 0,
        })?;
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
        let checkpoint = EvmAccessCheckpoint {
            address_len: self.address_len,
            storage_len: self.storage_len,
            generation: self.generation,
            depth,
        };
        self.checkpoint_depth = depth;
        Ok(checkpoint)
    }

    fn commit(&mut self, checkpoint: Self::Checkpoint) -> Result<(), EvmCoreError> {
        self.validate_checkpoint(&checkpoint)?;
        self.checkpoint_depth = self.checkpoint_depth.saturating_sub(1);
        Ok(())
    }

    fn revert(&mut self, checkpoint: Self::Checkpoint) -> Result<(), EvmCoreError> {
        self.validate_checkpoint(&checkpoint)?;
        self.restore(AccessCheckpoint {
            address_len: checkpoint.address_len,
            storage_len: checkpoint.storage_len,
        })?;
        self.checkpoint_depth = self.checkpoint_depth.saturating_sub(1);
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
        if self
            .addresses
            .get(..self.address_len)
            .is_some_and(|addresses| addresses.contains(&address))
        {
            return Ok(EvmAccessStatus::Warm);
        }
        let slot = self
            .addresses
            .get_mut(self.address_len)
            .ok_or(EvmCoreError::StateAccessListFull)?;
        *slot = address;
        self.address_len = self
            .address_len
            .checked_add(1)
            .ok_or(EvmCoreError::StateAccessListFull)?;
        Ok(EvmAccessStatus::Cold)
    }

    fn warm_storage(
        &mut self,
        address: EvmAddress,
        key: EvmWord,
    ) -> Result<EvmAccessStatus, EvmCoreError> {
        let access = StorageSlotAccess { address, key };
        if self
            .storage
            .get(..self.storage_len)
            .is_some_and(|storage| storage.contains(&access))
        {
            return Ok(EvmAccessStatus::Warm);
        }
        let address_is_warm = self
            .addresses
            .get(..self.address_len)
            .is_some_and(|addresses| addresses.contains(&address));
        if !address_is_warm && self.address_len >= ADDRESSES {
            return Err(EvmCoreError::StateAccessListFull);
        }
        if self.storage_len >= STORAGE {
            return Err(EvmCoreError::StateAccessListFull);
        }
        if !address_is_warm {
            let address_slot = self
                .addresses
                .get_mut(self.address_len)
                .ok_or(EvmCoreError::StateAccessListFull)?;
            *address_slot = address;
            self.address_len = self
                .address_len
                .checked_add(1)
                .ok_or(EvmCoreError::StateAccessListFull)?;
        }
        let storage_slot = self
            .storage
            .get_mut(self.storage_len)
            .ok_or(EvmCoreError::StateAccessListFull)?;
        *storage_slot = access;
        self.storage_len = self
            .storage_len
            .checked_add(1)
            .ok_or(EvmCoreError::StateAccessListFull)?;
        Ok(EvmAccessStatus::Cold)
    }
}

/// Compatibility name for the explicit embedded tracker.
pub type EvmAccessSet<const ADDRESSES: usize, const STORAGE: usize> =
    EvmEmbeddedAccessTracker<ADDRESSES, STORAGE>;
