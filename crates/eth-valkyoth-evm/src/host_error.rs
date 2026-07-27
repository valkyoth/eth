use core::fmt;

/// Host capability or bounded arena failure.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HostCapabilityError {
    /// Snapshot-pure state read failed.
    StateReadFailed,
    /// Journal lifecycle or mutation failed.
    JournalFailed,
    /// Warm-access tracking failed or reached capacity.
    AccessTrackingFailed,
    /// Cryptographic provider failed.
    CryptoFailed,
    /// Configured frame capacity is zero or exceeds the EVM limit.
    InvalidFrameCapacity,
    /// Iterative call depth reached configured capacity.
    CallDepthExceeded,
    /// No iterative call frame is active.
    CallFrameMissing,
    /// Requested memory exceeds the borrowed arena.
    MemoryCapacityExceeded,
    /// Host consistency is unknown after a failed multi-capability transition.
    HostPoisoned,
    /// Transaction-local capabilities were used before destructive reset.
    TransactionNotStarted,
}

impl HostCapabilityError {
    /// Stable machine-readable error code.
    #[must_use]
    pub const fn code(self) -> &'static str {
        match self {
            Self::StateReadFailed => "ETH_EVM_HOST_STATE_READ",
            Self::JournalFailed => "ETH_EVM_HOST_JOURNAL",
            Self::AccessTrackingFailed => "ETH_EVM_HOST_ACCESS_TRACKING",
            Self::CryptoFailed => "ETH_EVM_HOST_CRYPTO",
            Self::InvalidFrameCapacity => "ETH_EVM_HOST_INVALID_FRAME_CAPACITY",
            Self::CallDepthExceeded => "ETH_EVM_HOST_CALL_DEPTH",
            Self::CallFrameMissing => "ETH_EVM_HOST_CALL_FRAME_MISSING",
            Self::MemoryCapacityExceeded => "ETH_EVM_HOST_MEMORY_CAPACITY",
            Self::HostPoisoned => "ETH_EVM_HOST_POISONED",
            Self::TransactionNotStarted => "ETH_EVM_HOST_TRANSACTION_NOT_STARTED",
        }
    }

    /// Stable human-readable error message.
    #[must_use]
    pub const fn message(self) -> &'static str {
        match self {
            Self::StateReadFailed => "execution state view read failed",
            Self::JournalFailed => "execution state journal operation failed",
            Self::AccessTrackingFailed => "execution access tracker operation failed",
            Self::CryptoFailed => "execution cryptographic provider operation failed",
            Self::InvalidFrameCapacity => "iterative call-frame capacity is invalid",
            Self::CallDepthExceeded => "iterative EVM call depth exceeded",
            Self::CallFrameMissing => "iterative EVM call frame is missing",
            Self::MemoryCapacityExceeded => "transaction memory capacity exceeded",
            Self::HostPoisoned => "execution host consistency is unknown",
            Self::TransactionNotStarted => "execution host transaction was not started",
        }
    }
}

impl fmt::Display for HostCapabilityError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.message())
    }
}

#[cfg(feature = "std")]
impl std::error::Error for HostCapabilityError {}

/// Failure while creating a child journal checkpoint and iterative frame.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BeginChildError {
    /// The journal could not create the child checkpoint.
    CheckpointFailed(HostCapabilityError),
    /// Access checkpoint creation failed after the journal checkpoint.
    ///
    /// Access consistency cannot be proven after a mutable backend error, so
    /// the host is poisoned even when journal cleanup succeeds.
    AccessCheckpointFailed {
        /// Access-tracker checkpoint error.
        error: HostCapabilityError,
        /// Journal cleanup error, when cleanup also failed.
        journal_revert_error: Option<HostCapabilityError>,
    },
    /// The frame was rejected and the fresh checkpoint was reverted.
    FrameRejected {
        /// Arena error that rejected the frame.
        frame_error: HostCapabilityError,
    },
    /// The frame was rejected and cleanup of the fresh checkpoint also failed.
    ///
    /// Journal consistency is unknown after this error. The host must not
    /// continue or retry the transaction.
    FrameRejectedAndCleanupFailed {
        /// Arena error that rejected the frame.
        frame_error: HostCapabilityError,
        /// Access-tracker cleanup error, when access rollback failed.
        access_revert_error: Option<HostCapabilityError>,
        /// Journal cleanup error, when journal rollback failed.
        journal_revert_error: Option<HostCapabilityError>,
    },
}

impl BeginChildError {
    /// Stable machine-readable error code.
    #[must_use]
    pub const fn code(self) -> &'static str {
        match self {
            Self::CheckpointFailed(error) | Self::FrameRejected { frame_error: error } => {
                error.code()
            }
            Self::AccessCheckpointFailed { .. } => "ETH_EVM_HOST_ACCESS_CHECKPOINT_FAILED",
            Self::FrameRejectedAndCleanupFailed { .. } => {
                "ETH_EVM_HOST_FRAME_REJECTED_CLEANUP_FAILED"
            }
        }
    }

    /// Stable human-readable error message.
    #[must_use]
    pub const fn message(self) -> &'static str {
        match self {
            Self::CheckpointFailed(_) => "execution child checkpoint creation failed",
            Self::AccessCheckpointFailed { .. } => {
                "execution child access checkpoint creation failed"
            }
            Self::FrameRejected { .. } => "execution child frame was rejected",
            Self::FrameRejectedAndCleanupFailed { .. } => {
                "execution child frame was rejected and capability cleanup failed"
            }
        }
    }
}

impl fmt::Display for BeginChildError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.message())
    }
}

#[cfg(feature = "std")]
impl std::error::Error for BeginChildError {}

/// Journal action selected after executing one child frame.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ChildFinalizeAction {
    /// Retain journaled child changes.
    Commit,
    /// Roll back journaled child changes.
    Revert,
}

/// Failure while running or atomically finalizing one child lifecycle.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ChildLifecycleError {
    /// The host was already poisoned by an earlier consistency failure.
    HostPoisoned,
    /// The host transaction lifecycle was not started.
    TransactionNotStarted,
    /// The child checkpoint or frame could not be created.
    Begin(BeginChildError),
    /// Nested execution did not restore the exact child-frame depth.
    FrameDepthMismatch {
        /// Expected active frame count.
        expected: usize,
        /// Observed active frame count.
        found: usize,
    },
    /// Journal or access finalization failed after child execution.
    CapabilityConsistencyUnknown {
        /// Requested finalization action.
        action: ChildFinalizeAction,
        /// Journal failure, when journal finalization failed.
        journal_error: Option<HostCapabilityError>,
        /// Access failure, when access finalization failed.
        access_error: Option<HostCapabilityError>,
    },
    /// The journal finalized but the arena could not remove the matching frame.
    ArenaConsistencyUnknown {
        /// Completed journal action.
        action: ChildFinalizeAction,
        /// Arena failure.
        error: HostCapabilityError,
    },
    /// The arena removed a frame other than the one that started this child.
    FrameMismatch {
        /// Completed journal action.
        action: ChildFinalizeAction,
    },
}

impl ChildLifecycleError {
    /// Stable machine-readable error code.
    #[must_use]
    pub const fn code(self) -> &'static str {
        match self {
            Self::HostPoisoned => "ETH_EVM_HOST_POISONED",
            Self::TransactionNotStarted => "ETH_EVM_HOST_TRANSACTION_NOT_STARTED",
            Self::Begin(error) => error.code(),
            Self::FrameDepthMismatch { .. } => "ETH_EVM_HOST_CHILD_DEPTH_MISMATCH",
            Self::CapabilityConsistencyUnknown { .. } => {
                "ETH_EVM_HOST_CHILD_CAPABILITY_CONSISTENCY_UNKNOWN"
            }
            Self::ArenaConsistencyUnknown { .. } => "ETH_EVM_HOST_CHILD_ARENA_CONSISTENCY_UNKNOWN",
            Self::FrameMismatch { .. } => "ETH_EVM_HOST_CHILD_FRAME_MISMATCH",
        }
    }

    /// Stable human-readable error message.
    #[must_use]
    pub const fn message(self) -> &'static str {
        match self {
            Self::HostPoisoned => "execution host is poisoned",
            Self::TransactionNotStarted => "execution host transaction was not started",
            Self::Begin(_) => "execution child could not start",
            Self::FrameDepthMismatch { .. } => {
                "execution child nesting did not restore the expected depth"
            }
            Self::CapabilityConsistencyUnknown { .. } => {
                "execution child capability consistency is unknown"
            }
            Self::ArenaConsistencyUnknown { .. } => "execution child arena consistency is unknown",
            Self::FrameMismatch { .. } => "execution child removed a different iterative frame",
        }
    }
}

impl fmt::Display for ChildLifecycleError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.message())
    }
}

#[cfg(feature = "std")]
impl std::error::Error for ChildLifecycleError {}
