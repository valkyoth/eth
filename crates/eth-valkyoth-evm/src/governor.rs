use core::fmt;

use crate::{MAX_ITERATIVE_CALL_FRAMES, MAX_TRANSACTION_MEMORY_BYTES};
use eth_valkyoth_evm_core::{EVM_MAX_WARM_ADDRESSES, EVM_MAX_WARM_STORAGE_SLOTS};

/// Maximum transaction-local journal entries admitted by the bootstrap policy.
pub const MAX_TRANSACTION_JOURNAL_ENTRIES: usize = 10_000_000;
/// Maximum transaction-local journal checkpoints admitted by the EVM.
pub const MAX_TRANSACTION_JOURNAL_CHECKPOINTS: usize = MAX_ITERATIVE_CALL_FRAMES;
/// Maximum reusable arena bytes admitted by the bootstrap policy.
pub const MAX_REUSABLE_ARENA_BYTES: usize = 67_108_864;
/// Maximum transaction-local cache entries admitted by the bootstrap policy.
pub const MAX_TRANSACTION_CACHE_ENTRIES: usize = 1_000_000;

/// Requested execution-resource limits validated at a deployment boundary.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ExecutionResourceRequest {
    /// Distinct warm-address capacity.
    pub warm_addresses: usize,
    /// Distinct warm-storage capacity.
    pub warm_storage_slots: usize,
    /// Journal-entry capacity.
    pub journal_entries: usize,
    /// Journal-checkpoint capacity.
    pub journal_checkpoints: usize,
    /// Iterative frame capacity.
    pub frames: usize,
    /// Visible EVM memory capacity in bytes.
    pub memory_bytes: usize,
    /// Retained reusable arena capacity in bytes.
    pub reusable_arena_bytes: usize,
    /// Retained cache-entry capacity.
    pub cache_entries: usize,
    /// Abstract execution-work capacity.
    pub work_units: u64,
}

/// Validated immutable resource limits for one execution governor.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ExecutionResourceLimits {
    request: ExecutionResourceRequest,
}

impl ExecutionResourceLimits {
    /// Validates nonzero capacities against protocol and bootstrap ceilings.
    pub const fn try_new(
        request: ExecutionResourceRequest,
    ) -> Result<Self, ExecutionGovernorError> {
        if request.warm_addresses == 0
            || request.warm_storage_slots == 0
            || request.journal_entries == 0
            || request.journal_checkpoints == 0
            || request.frames == 0
            || request.memory_bytes == 0
            || request.reusable_arena_bytes == 0
            || request.cache_entries == 0
            || request.work_units == 0
        {
            return Err(ExecutionGovernorError::ZeroLimit);
        }
        if request.warm_addresses > EVM_MAX_WARM_ADDRESSES
            || request.warm_storage_slots > EVM_MAX_WARM_STORAGE_SLOTS
            || request.journal_entries > MAX_TRANSACTION_JOURNAL_ENTRIES
            || request.journal_checkpoints > MAX_TRANSACTION_JOURNAL_CHECKPOINTS
            || request.frames > MAX_ITERATIVE_CALL_FRAMES
            || request.memory_bytes > MAX_TRANSACTION_MEMORY_BYTES
            || request.reusable_arena_bytes > MAX_REUSABLE_ARENA_BYTES
            || request.cache_entries > MAX_TRANSACTION_CACHE_ENTRIES
        {
            return Err(ExecutionGovernorError::LimitTooLarge);
        }
        Ok(Self { request })
    }

    /// Returns the validated request.
    #[must_use]
    pub const fn request(self) -> ExecutionResourceRequest {
        self.request
    }
}

/// Resource class charged monotonically during one transaction.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ExecutionResource {
    /// Distinct warm addresses.
    WarmAddress,
    /// Distinct warm storage slots.
    WarmStorageSlot,
    /// Journal entries.
    JournalEntry,
    /// Journal checkpoints.
    JournalCheckpoint,
    /// Iterative frames entered.
    Frame,
    /// Visible EVM memory bytes.
    MemoryByte,
    /// Reusable arena bytes retained.
    ReusableArenaByte,
    /// Cache entries retained.
    CacheEntry,
}

/// Bounded transaction-local execution-resource governor.
#[derive(Debug, Eq, PartialEq)]
pub struct ExecutionGovernor {
    limits: ExecutionResourceLimits,
    used: ExecutionResourceUsage,
    work_remaining: u64,
    generation: u64,
    transaction_started: bool,
}

#[derive(Debug, Eq, PartialEq)]
struct ExecutionResourceUsage {
    warm_addresses: usize,
    warm_storage_slots: usize,
    journal_entries: usize,
    journal_checkpoints: usize,
    frames: usize,
    memory_bytes: usize,
    reusable_arena_bytes: usize,
    cache_entries: usize,
}

impl ExecutionResourceUsage {
    const ZERO: Self = Self {
        warm_addresses: 0,
        warm_storage_slots: 0,
        journal_entries: 0,
        journal_checkpoints: 0,
        frames: 0,
        memory_bytes: 0,
        reusable_arena_bytes: 0,
        cache_entries: 0,
    };

    const fn get(&self, resource: ExecutionResource) -> usize {
        match resource {
            ExecutionResource::WarmAddress => self.warm_addresses,
            ExecutionResource::WarmStorageSlot => self.warm_storage_slots,
            ExecutionResource::JournalEntry => self.journal_entries,
            ExecutionResource::JournalCheckpoint => self.journal_checkpoints,
            ExecutionResource::Frame => self.frames,
            ExecutionResource::MemoryByte => self.memory_bytes,
            ExecutionResource::ReusableArenaByte => self.reusable_arena_bytes,
            ExecutionResource::CacheEntry => self.cache_entries,
        }
    }

    fn set(&mut self, resource: ExecutionResource, value: usize) {
        match resource {
            ExecutionResource::WarmAddress => self.warm_addresses = value,
            ExecutionResource::WarmStorageSlot => self.warm_storage_slots = value,
            ExecutionResource::JournalEntry => self.journal_entries = value,
            ExecutionResource::JournalCheckpoint => self.journal_checkpoints = value,
            ExecutionResource::Frame => self.frames = value,
            ExecutionResource::MemoryByte => self.memory_bytes = value,
            ExecutionResource::ReusableArenaByte => self.reusable_arena_bytes = value,
            ExecutionResource::CacheEntry => self.cache_entries = value,
        }
    }
}

impl ExecutionGovernor {
    /// Creates an empty governor with validated immutable limits.
    #[must_use]
    pub const fn new(limits: ExecutionResourceLimits) -> Self {
        Self {
            work_remaining: limits.request.work_units,
            limits,
            used: ExecutionResourceUsage::ZERO,
            generation: 0,
            transaction_started: false,
        }
    }

    /// Destructively starts a fresh transaction budget.
    pub fn reset_transaction(&mut self) -> Result<(), ExecutionGovernorError> {
        self.used = ExecutionResourceUsage::ZERO;
        self.work_remaining = self.limits.request.work_units;
        self.generation = self
            .generation
            .checked_add(1)
            .ok_or(ExecutionGovernorError::GenerationExhausted)?;
        self.transaction_started = true;
        Ok(())
    }

    /// Charges one resource class; failed and cancelled work is not refunded.
    pub fn charge(
        &mut self,
        resource: ExecutionResource,
        amount: usize,
    ) -> Result<(), ExecutionGovernorError> {
        if amount == 0 {
            return Err(ExecutionGovernorError::ZeroCharge);
        }
        if !self.transaction_started {
            return Err(ExecutionGovernorError::TransactionNotStarted);
        }
        let next = self
            .used
            .get(resource)
            .checked_add(amount)
            .ok_or(ExecutionGovernorError::ResourceExhausted(resource))?;
        if next > self.limit(resource) {
            return Err(ExecutionGovernorError::ResourceExhausted(resource));
        }
        self.used.set(resource, next);
        Ok(())
    }

    /// Records a simultaneous or retained high-water requirement.
    ///
    /// Use this for frame/checkpoint depth, visible memory, arenas, and caches
    /// whose capacity may be reused. A lower later observation does not refund
    /// the transaction's recorded high-water mark.
    pub fn observe_capacity(
        &mut self,
        resource: ExecutionResource,
        required: usize,
    ) -> Result<(), ExecutionGovernorError> {
        if required == 0 {
            return Err(ExecutionGovernorError::ZeroCharge);
        }
        if !self.transaction_started {
            return Err(ExecutionGovernorError::TransactionNotStarted);
        }
        if required > self.limit(resource) {
            return Err(ExecutionGovernorError::ResourceExhausted(resource));
        }
        if required > self.used.get(resource) {
            self.used.set(resource, required);
        }
        Ok(())
    }

    /// Issues a non-forgeable root work token and permanently charges it.
    pub fn issue_work(&mut self, units: u64) -> Result<ExecutionWorkToken, ExecutionGovernorError> {
        if units == 0 {
            return Err(ExecutionGovernorError::ZeroCharge);
        }
        if !self.transaction_started {
            return Err(ExecutionGovernorError::TransactionNotStarted);
        }
        self.work_remaining = self
            .work_remaining
            .checked_sub(units)
            .ok_or(ExecutionGovernorError::WorkExhausted)?;
        Ok(ExecutionWorkToken {
            units,
            generation: self.generation,
        })
    }

    /// Returns cumulative usage or the recorded high-water mark.
    #[must_use]
    pub const fn used(&self, resource: ExecutionResource) -> usize {
        self.used.get(resource)
    }

    /// Returns unissued work units.
    #[must_use]
    pub const fn work_remaining(&self) -> u64 {
        self.work_remaining
    }

    /// Returns the immutable limits.
    #[must_use]
    pub const fn limits(&self) -> ExecutionResourceLimits {
        self.limits
    }

    const fn limit(&self, resource: ExecutionResource) -> usize {
        let request = self.limits.request;
        match resource {
            ExecutionResource::WarmAddress => request.warm_addresses,
            ExecutionResource::WarmStorageSlot => request.warm_storage_slots,
            ExecutionResource::JournalEntry => request.journal_entries,
            ExecutionResource::JournalCheckpoint => request.journal_checkpoints,
            ExecutionResource::Frame => request.frames,
            ExecutionResource::MemoryByte => request.memory_bytes,
            ExecutionResource::ReusableArenaByte => request.reusable_arena_bytes,
            ExecutionResource::CacheEntry => request.cache_entries,
        }
    }
}

/// Hierarchical, non-copyable execution-work authority.
#[derive(Debug, Eq, PartialEq)]
pub struct ExecutionWorkToken {
    units: u64,
    generation: u64,
}

impl ExecutionWorkToken {
    /// Delegates part of this token to a child without creating new authority.
    pub fn delegate(&mut self, units: u64) -> Result<Self, ExecutionGovernorError> {
        if units == 0 {
            return Err(ExecutionGovernorError::ZeroCharge);
        }
        self.units = self
            .units
            .checked_sub(units)
            .ok_or(ExecutionGovernorError::InvalidDelegation)?;
        Ok(Self {
            units,
            generation: self.generation,
        })
    }

    /// Returns the work authority remaining in this token.
    #[must_use]
    pub const fn units(&self) -> u64 {
        self.units
    }

    /// Returns the transaction generation that issued this token.
    #[must_use]
    pub const fn generation(&self) -> u64 {
        self.generation
    }
}

/// Resource-governor validation or exhaustion failure.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ExecutionGovernorError {
    /// A configured limit was zero.
    ZeroLimit,
    /// A configured limit exceeded its reviewed hard ceiling.
    LimitTooLarge,
    /// A zero-sized charge or delegation was requested.
    ZeroCharge,
    /// Resource authority was used before destructive transaction reset.
    TransactionNotStarted,
    /// One named resource was exhausted.
    ResourceExhausted(ExecutionResource),
    /// Abstract work authority was exhausted.
    WorkExhausted,
    /// A child requested more authority than its parent held.
    InvalidDelegation,
    /// Transaction generations can no longer be represented.
    GenerationExhausted,
}

impl ExecutionGovernorError {
    /// Stable machine-readable error code.
    #[must_use]
    pub const fn code(self) -> &'static str {
        match self {
            Self::ZeroLimit => "ETH_EVM_GOVERNOR_ZERO_LIMIT",
            Self::LimitTooLarge => "ETH_EVM_GOVERNOR_LIMIT_TOO_LARGE",
            Self::ZeroCharge => "ETH_EVM_GOVERNOR_ZERO_CHARGE",
            Self::TransactionNotStarted => "ETH_EVM_GOVERNOR_TRANSACTION_NOT_STARTED",
            Self::ResourceExhausted(_) => "ETH_EVM_GOVERNOR_RESOURCE_EXHAUSTED",
            Self::WorkExhausted => "ETH_EVM_GOVERNOR_WORK_EXHAUSTED",
            Self::InvalidDelegation => "ETH_EVM_GOVERNOR_INVALID_DELEGATION",
            Self::GenerationExhausted => "ETH_EVM_GOVERNOR_GENERATION_EXHAUSTED",
        }
    }
}

impl fmt::Display for ExecutionGovernorError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.code())
    }
}

#[cfg(feature = "std")]
impl std::error::Error for ExecutionGovernorError {}
