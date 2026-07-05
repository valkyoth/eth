use core::fmt;

use eth_valkyoth_primitives::{Address, B256, Nonce, Wei};

/// Account data visible to an execution backend.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SnapshotAccount {
    /// Account nonce.
    pub nonce: Nonce,
    /// Account balance.
    pub balance: Wei,
    /// Hash of the account bytecode.
    pub code_hash: B256,
}

/// Explicit state snapshot boundary used by execution requests.
pub trait StateSnapshot {
    /// Stable caller-provided identifier for the state snapshot.
    fn snapshot_id(&self) -> B256;

    /// Returns account state for an address if it exists in the snapshot.
    fn account(&self, address: Address) -> Result<Option<SnapshotAccount>, SnapshotError>;

    /// Returns a storage slot value for an account.
    fn storage(&self, address: Address, slot: B256) -> Result<B256, SnapshotError>;
}

/// State snapshot access failure.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SnapshotError {
    /// Snapshot data is unavailable.
    Unavailable,
    /// Snapshot data does not match the requested state identity.
    SnapshotMismatch,
    /// Storage was requested for an absent account.
    AccountMissing,
}

impl SnapshotError {
    /// Stable machine-readable error code.
    #[must_use]
    pub const fn code(self) -> &'static str {
        match self {
            Self::Unavailable => "ETH_EVM_SNAPSHOT_UNAVAILABLE",
            Self::SnapshotMismatch => "ETH_EVM_SNAPSHOT_MISMATCH",
            Self::AccountMissing => "ETH_EVM_SNAPSHOT_ACCOUNT_MISSING",
        }
    }

    /// Stable human-readable error message.
    #[must_use]
    pub const fn message(self) -> &'static str {
        match self {
            Self::Unavailable => "state snapshot data is unavailable",
            Self::SnapshotMismatch => "state snapshot identity does not match",
            Self::AccountMissing => "state snapshot account is missing",
        }
    }
}

impl fmt::Display for SnapshotError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.message())
    }
}

#[cfg(feature = "std")]
impl std::error::Error for SnapshotError {}
