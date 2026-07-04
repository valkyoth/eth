use eth_valkyoth_primitives::B256;

/// Generic MPT proof root hash domain.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MptProofRoot(B256);

impl MptProofRoot {
    /// Creates a proof root from raw hash bytes.
    #[must_use]
    pub const fn from_b256(value: B256) -> Self {
        Self(value)
    }

    /// Returns the raw root hash.
    #[must_use]
    pub const fn to_b256(self) -> B256 {
        self.0
    }
}

/// Transaction trie root hash domain.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TransactionTrieRoot(B256);

impl TransactionTrieRoot {
    /// Creates a transaction trie root from raw hash bytes.
    #[must_use]
    pub const fn from_b256(value: B256) -> Self {
        Self(value)
    }

    /// Returns the raw root hash.
    #[must_use]
    pub const fn to_b256(self) -> B256 {
        self.0
    }
}

impl From<TransactionTrieRoot> for MptProofRoot {
    fn from(value: TransactionTrieRoot) -> Self {
        Self(value.to_b256())
    }
}

/// Receipt trie root hash domain.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ReceiptTrieRoot(B256);

impl ReceiptTrieRoot {
    /// Creates a receipt trie root from raw hash bytes.
    #[must_use]
    pub const fn from_b256(value: B256) -> Self {
        Self(value)
    }

    /// Returns the raw root hash.
    #[must_use]
    pub const fn to_b256(self) -> B256 {
        self.0
    }
}

impl From<ReceiptTrieRoot> for MptProofRoot {
    fn from(value: ReceiptTrieRoot) -> Self {
        Self(value.to_b256())
    }
}

/// Successful transaction inclusion proof.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct VerifiedTransactionInclusion {
    index: u64,
    root: TransactionTrieRoot,
}

impl VerifiedTransactionInclusion {
    pub(crate) const fn new(index: u64, root: TransactionTrieRoot) -> Self {
        Self { index, root }
    }

    /// Returns the transaction index proven in the trie.
    #[must_use]
    pub const fn index(self) -> u64 {
        self.index
    }

    /// Returns the transaction root used for verification.
    #[must_use]
    pub const fn root(self) -> TransactionTrieRoot {
        self.root
    }
}

/// Successful receipt inclusion proof.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct VerifiedReceiptInclusion {
    index: u64,
    root: ReceiptTrieRoot,
}

impl VerifiedReceiptInclusion {
    pub(crate) const fn new(index: u64, root: ReceiptTrieRoot) -> Self {
        Self { index, root }
    }

    /// Returns the receipt index proven in the trie.
    #[must_use]
    pub const fn index(self) -> u64 {
        self.index
    }

    /// Returns the receipt root used for verification.
    #[must_use]
    pub const fn root(self) -> ReceiptTrieRoot {
        self.root
    }
}
