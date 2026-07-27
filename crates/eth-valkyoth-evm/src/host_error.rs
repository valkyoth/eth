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
    /// The frame was rejected and the fresh checkpoint was reverted.
    FrameRejected {
        /// Arena error that rejected the frame.
        frame_error: HostCapabilityError,
    },
    /// The frame was rejected and cleanup of the fresh checkpoint also failed.
    ///
    /// Journal consistency is unknown after this error. The host must not
    /// continue or retry the transaction.
    FrameRejectedAndJournalRevertFailed {
        /// Arena error that rejected the frame.
        frame_error: HostCapabilityError,
        /// Journal error that prevented checkpoint cleanup.
        revert_error: HostCapabilityError,
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
            Self::FrameRejectedAndJournalRevertFailed { .. } => {
                "ETH_EVM_HOST_FRAME_REJECTED_JOURNAL_REVERT_FAILED"
            }
        }
    }

    /// Stable human-readable error message.
    #[must_use]
    pub const fn message(self) -> &'static str {
        match self {
            Self::CheckpointFailed(_) => "execution child checkpoint creation failed",
            Self::FrameRejected { .. } => "execution child frame was rejected",
            Self::FrameRejectedAndJournalRevertFailed { .. } => {
                "execution child frame was rejected and journal cleanup failed"
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
