use crate::{EvmAddress, EvmCoreError, EvmWord};

/// Maximum number of active EVM call frames, including the root frame.
pub const EVM_CALL_DEPTH_LIMIT: u16 = 1024;

/// Call-family opcode kind.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EvmCallKind {
    /// `CALL`.
    Call,
    /// `CALLCODE`.
    CallCode,
    /// `DELEGATECALL`.
    DelegateCall,
    /// `STATICCALL`.
    StaticCall,
}

impl EvmCallKind {
    /// Returns whether the kind has an explicit value stack operand.
    #[must_use]
    pub const fn has_value_operand(self) -> bool {
        matches!(self, Self::Call | Self::CallCode)
    }

    /// Returns whether this kind forces the child frame static flag.
    #[must_use]
    pub const fn forces_static(self) -> bool {
        matches!(self, Self::StaticCall)
    }
}

/// Create-family opcode kind.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EvmCreateKind {
    /// `CREATE`.
    Create,
    /// `CREATE2`.
    Create2,
}

/// Memory range used by call input/output and return-data handling.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EvmMemoryRange {
    /// Offset in the current memory view.
    pub offset: usize,
    /// Length in bytes.
    pub len: usize,
}

impl EvmMemoryRange {
    /// Constructs a memory range after checking offset + length overflow.
    pub const fn try_new(offset: usize, len: usize) -> Result<Self, EvmCoreError> {
        match offset.checked_add(len) {
            Some(_) => Ok(Self { offset, len }),
            None => Err(EvmCoreError::ReturnDataOutOfBounds),
        }
    }
}

/// Execution-frame policy for call/create planning.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EvmCallFramePolicy {
    depth: u16,
    is_static: bool,
}

impl EvmCallFramePolicy {
    /// Constructs the root frame policy.
    #[must_use]
    pub const fn root() -> Self {
        Self {
            depth: 0,
            is_static: false,
        }
    }

    /// Constructs a frame policy for tests and explicit embeddings.
    pub const fn try_new(depth: u16, is_static: bool) -> Result<Self, EvmCoreError> {
        if depth >= EVM_CALL_DEPTH_LIMIT {
            return Err(EvmCoreError::CallDepthLimitReached);
        }
        Ok(Self { depth, is_static })
    }

    /// Returns the zero-based current call depth.
    #[must_use]
    pub const fn depth(self) -> u16 {
        self.depth
    }

    /// Returns whether this frame is static.
    #[must_use]
    pub const fn is_static(self) -> bool {
        self.is_static
    }

    /// Returns the policy for a child call frame.
    pub fn enter_call(self, kind: EvmCallKind, value: EvmWord) -> Result<Self, EvmCoreError> {
        if self.is_static && matches!(kind, EvmCallKind::Call) && !value.is_zero() {
            return Err(EvmCoreError::StaticStateChange);
        }
        let depth = match self.depth.checked_add(1) {
            Some(depth) if depth < EVM_CALL_DEPTH_LIMIT => depth,
            _ => return Err(EvmCoreError::CallDepthLimitReached),
        };
        Ok(Self {
            depth,
            is_static: self.is_static || kind.forces_static(),
        })
    }

    /// Returns the policy for a child create frame.
    pub fn enter_create(self, _kind: EvmCreateKind) -> Result<Self, EvmCoreError> {
        match self.ensure_state_write_allowed() {
            Ok(()) => {}
            Err(error) => return Err(error),
        }
        let depth = match self.depth.checked_add(1) {
            Some(depth) if depth < EVM_CALL_DEPTH_LIMIT => depth,
            _ => return Err(EvmCoreError::CallDepthLimitReached),
        };
        Ok(Self {
            depth,
            is_static: self.is_static,
        })
    }

    /// Fails when a state-changing operation is attempted in a static frame.
    pub const fn ensure_state_write_allowed(self) -> Result<(), EvmCoreError> {
        if self.is_static {
            return Err(EvmCoreError::StaticStateChange);
        }
        Ok(())
    }
}

/// Parsed call plan. It is validation metadata, not executable host behavior.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EvmCallPlan {
    /// Call opcode kind.
    pub kind: EvmCallKind,
    /// Requested callee gas.
    pub gas: EvmWord,
    /// Target address.
    pub target: EvmAddress,
    /// Explicit call value, or zero for value-less call kinds.
    pub value: EvmWord,
    /// Input calldata memory range.
    pub input: EvmMemoryRange,
    /// Output scratch memory range.
    pub output: EvmMemoryRange,
    /// Child frame policy produced by this call.
    pub child_frame: EvmCallFramePolicy,
}

impl EvmCallPlan {
    /// Constructs a validated call plan.
    pub fn try_new(
        kind: EvmCallKind,
        gas: EvmWord,
        target: EvmAddress,
        value: EvmWord,
        input: EvmMemoryRange,
        output: EvmMemoryRange,
        frame: EvmCallFramePolicy,
    ) -> Result<Self, EvmCoreError> {
        let child_frame = frame.enter_call(kind, value)?;
        Ok(Self {
            kind,
            gas,
            target,
            value,
            input,
            output,
            child_frame,
        })
    }
}

/// Parsed create plan. It is validation metadata, not executable host behavior.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EvmCreatePlan {
    /// Create opcode kind.
    pub kind: EvmCreateKind,
    /// Endowment sent to the created account.
    pub value: EvmWord,
    /// Init-code memory range.
    pub init_code: EvmMemoryRange,
    /// CREATE2 salt, or zero for CREATE.
    pub salt: EvmWord,
    /// Child frame policy produced by this create.
    pub child_frame: EvmCallFramePolicy,
}

impl EvmCreatePlan {
    /// Constructs a validated create plan.
    pub fn try_new(
        kind: EvmCreateKind,
        value: EvmWord,
        init_code: EvmMemoryRange,
        salt: EvmWord,
        frame: EvmCallFramePolicy,
    ) -> Result<Self, EvmCoreError> {
        let child_frame = frame.enter_create(kind)?;
        Ok(Self {
            kind,
            value,
            init_code,
            salt,
            child_frame,
        })
    }
}

/// Parsed call/create plan used by the fail-closed interpreter boundary.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum EvmCallCreatePlan {
    /// Parsed call-family plan.
    Call(EvmCallPlan),
    /// Parsed create-family plan.
    Create(EvmCreatePlan),
}

/// Return-data range retained after a child call.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EvmReturnDataRange(EvmMemoryRange);

impl EvmReturnDataRange {
    /// Constructs a return-data range.
    pub const fn try_new(offset: usize, len: usize) -> Result<Self, EvmCoreError> {
        match EvmMemoryRange::try_new(offset, len) {
            Ok(range) => Ok(Self(range)),
            Err(error) => Err(error),
        }
    }

    /// Returns the underlying memory range.
    #[must_use]
    pub const fn range(self) -> EvmMemoryRange {
        self.0
    }

    /// Checks a bounded copy out of this return-data range.
    pub const fn check_copy(self, offset: usize, len: usize) -> Result<(), EvmCoreError> {
        let end = match offset.checked_add(len) {
            Some(end) => end,
            None => return Err(EvmCoreError::ReturnDataOutOfBounds),
        };
        if end > self.0.len {
            return Err(EvmCoreError::ReturnDataOutOfBounds);
        }
        Ok(())
    }
}

/// Journal checkpoint token.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EvmJournalCheckpoint {
    depth: usize,
}

/// Fixed-capacity journal checkpoint policy.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EvmJournal<const CHECKPOINTS: usize> {
    depth: usize,
}

impl<const CHECKPOINTS: usize> EvmJournal<CHECKPOINTS> {
    /// Constructs an empty journal checkpoint stack.
    pub const fn try_new() -> Result<Self, EvmCoreError> {
        if CHECKPOINTS == 0 {
            return Err(EvmCoreError::JournalCapacityZero);
        }
        Ok(Self { depth: 0 })
    }

    /// Returns the number of active checkpoints.
    #[must_use]
    pub const fn depth(self) -> usize {
        self.depth
    }

    /// Starts a checkpoint.
    pub fn begin(&mut self) -> Result<EvmJournalCheckpoint, EvmCoreError> {
        if self.depth >= CHECKPOINTS {
            return Err(EvmCoreError::JournalCheckpointOverflow);
        }
        let checkpoint = EvmJournalCheckpoint { depth: self.depth };
        self.depth = self
            .depth
            .checked_add(1)
            .ok_or(EvmCoreError::JournalCheckpointOverflow)?;
        Ok(checkpoint)
    }

    /// Commits the active checkpoint.
    pub fn commit(&mut self, checkpoint: EvmJournalCheckpoint) -> Result<(), EvmCoreError> {
        self.close(checkpoint)
    }

    /// Reverts the active checkpoint.
    pub fn revert(&mut self, checkpoint: EvmJournalCheckpoint) -> Result<(), EvmCoreError> {
        self.close(checkpoint)
    }

    fn close(&mut self, checkpoint: EvmJournalCheckpoint) -> Result<(), EvmCoreError> {
        let active = self
            .depth
            .checked_sub(1)
            .ok_or(EvmCoreError::JournalCheckpointMissing)?;
        if checkpoint.depth != active {
            return Err(EvmCoreError::JournalCheckpointMismatch);
        }
        self.depth = active;
        Ok(())
    }
}
