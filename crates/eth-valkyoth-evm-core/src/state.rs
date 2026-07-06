use crate::{EVM_MAX_BYTECODE_LEN, EvmCoreError, EvmWord};

const WORD_ADDRESS_OFFSET: usize = 12;

/// Ethereum address used by state-access opcodes.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct EvmAddress([u8; Self::LEN]);

impl EvmAddress {
    /// Address byte length.
    pub const LEN: usize = 20;
    /// Zero address.
    pub const ZERO: Self = Self([0u8; Self::LEN]);

    /// Constructs an address from canonical bytes.
    #[must_use]
    pub const fn from_bytes(bytes: [u8; Self::LEN]) -> Self {
        Self(bytes)
    }

    /// Converts a stack word to an address by taking the low 160 bits.
    #[must_use]
    pub fn from_word(word: EvmWord) -> Self {
        let bytes = word.to_be_bytes();
        let mut address = [0u8; Self::LEN];
        for (slot, source) in address
            .iter_mut()
            .zip(bytes.iter().skip(WORD_ADDRESS_OFFSET))
        {
            *slot = *source;
        }
        Self(address)
    }

    /// Returns the address bytes.
    #[must_use]
    pub const fn to_bytes(self) -> [u8; Self::LEN] {
        self.0
    }

    /// Converts this address to a left-padded EVM word.
    #[must_use]
    pub fn to_word(self) -> EvmWord {
        let mut bytes = [0u8; EvmWord::LEN];
        for (slot, source) in bytes
            .iter_mut()
            .skip(WORD_ADDRESS_OFFSET)
            .zip(self.0.iter())
        {
            *slot = *source;
        }
        EvmWord::from_be_bytes(bytes)
    }
}

/// Bounded account metadata returned by a host state snapshot.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EvmAccount {
    /// Whether the account exists in the queried state.
    pub exists: bool,
    /// Account balance.
    pub balance: EvmWord,
    /// Account code hash, or zero when the account does not exist.
    pub code_hash: EvmWord,
    /// Account code length in bytes.
    pub code_len: usize,
}

impl EvmAccount {
    /// Constructs account metadata with the release code-size cap enforced.
    pub const fn try_new(
        exists: bool,
        balance: EvmWord,
        code_hash: EvmWord,
        code_len: usize,
    ) -> Result<Self, EvmCoreError> {
        if code_len > EVM_MAX_BYTECODE_LEN {
            return Err(EvmCoreError::StateCodeTooLarge);
        }
        Ok(Self {
            exists,
            balance,
            code_hash,
            code_len,
        })
    }
}

/// Execution-local state context.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EvmStateContext {
    /// Address of the currently executing account.
    pub address: EvmAddress,
}

impl EvmStateContext {
    /// Constructs a state context.
    #[must_use]
    pub const fn new(address: EvmAddress) -> Self {
        Self { address }
    }
}

/// Host state snapshot used by explicit state-access execution.
pub trait EvmState {
    /// Returns account metadata for `address`.
    fn account(&mut self, address: EvmAddress) -> Result<EvmAccount, EvmCoreError>;

    /// Returns the storage word for `address` and `key`.
    fn storage(&mut self, address: EvmAddress, key: EvmWord) -> Result<EvmWord, EvmCoreError>;

    /// Copies code bytes into `output`, zero-padding bytes beyond available code.
    fn copy_code(
        &mut self,
        address: EvmAddress,
        code_offset: usize,
        output: &mut [u8],
    ) -> Result<(), EvmCoreError>;
}

/// Warm/cold access classification for one execution attempt.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EvmAccessStatus {
    /// The account or storage slot was already warm.
    Warm,
    /// The account or storage slot was cold and was marked warm by this access.
    Cold,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct StorageSlotAccess {
    address: EvmAddress,
    key: EvmWord,
}

/// Fixed-capacity warm access tracker.
///
/// Membership checks are linear scans over caller-chosen fixed-capacity
/// arrays. Keep capacities bounded relative to the execution gas limit and
/// deployment policy; this type deliberately avoids allocation and hashing.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EvmAccessSet<const ADDRESSES: usize, const STORAGE: usize> {
    addresses: [EvmAddress; ADDRESSES],
    address_len: usize,
    storage: [StorageSlotAccess; STORAGE],
    storage_len: usize,
}

impl<const ADDRESSES: usize, const STORAGE: usize> EvmAccessSet<ADDRESSES, STORAGE> {
    /// Creates an empty access set.
    pub const fn try_new() -> Result<Self, EvmCoreError> {
        if ADDRESSES == 0 {
            return Err(EvmCoreError::StateAccessListTooSmall);
        }
        if STORAGE == 0 {
            return Err(EvmCoreError::StateAccessListTooSmall);
        }
        Ok(Self {
            addresses: [EvmAddress::ZERO; ADDRESSES],
            address_len: 0,
            storage: [StorageSlotAccess {
                address: EvmAddress::ZERO,
                key: EvmWord::ZERO,
            }; STORAGE],
            storage_len: 0,
        })
    }

    /// Returns the number of warmed addresses.
    #[must_use]
    pub const fn address_len(self) -> usize {
        self.address_len
    }

    /// Returns the number of warmed storage slots.
    #[must_use]
    pub const fn storage_len(self) -> usize {
        self.storage_len
    }

    /// Marks an address warm and returns its previous access status.
    pub fn warm_address(&mut self, address: EvmAddress) -> Result<EvmAccessStatus, EvmCoreError> {
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

    /// Marks a storage slot warm and returns its previous access status.
    pub fn warm_storage(
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
        if !address_is_warm {
            let _ = self
                .addresses
                .get(self.address_len)
                .ok_or(EvmCoreError::StateAccessListFull)?;
        }
        let _ = self
            .storage
            .get(self.storage_len)
            .ok_or(EvmCoreError::StateAccessListFull)?;
        if !address_is_warm {
            let slot = self
                .addresses
                .get_mut(self.address_len)
                .ok_or(EvmCoreError::StateAccessListFull)?;
            *slot = address;
            self.address_len = self
                .address_len
                .checked_add(1)
                .ok_or(EvmCoreError::StateAccessListFull)?;
        }
        let slot = self
            .storage
            .get_mut(self.storage_len)
            .ok_or(EvmCoreError::StateAccessListFull)?;
        *slot = access;
        self.storage_len = self
            .storage_len
            .checked_add(1)
            .ok_or(EvmCoreError::StateAccessListFull)?;
        Ok(EvmAccessStatus::Cold)
    }
}
