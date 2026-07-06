use crate::{EvmCoreError, EvmFork, EvmOpcode};

/// Default gas limit for local deterministic execution tests.
pub const EVM_DEFAULT_GAS_LIMIT: u64 = 30_000_000;
/// Hard maximum gas limit accepted by the bootstrap interpreter.
pub const EVM_MAX_GAS_LIMIT: u64 = 1_000_000_000;
const EVM_MEMORY_WORD_BYTES: usize = 32;
const EVM_MEMORY_QUADRATIC_DIVISOR: u64 = 512;
const EVM_COPY_GAS_WORD_BYTES: usize = 32;

/// Bounded gas amount used by the native EVM core.
#[derive(Clone, Copy, Debug, Default, Eq, Ord, PartialEq, PartialOrd)]
pub struct EvmGas(u64);

impl EvmGas {
    /// Constructs a gas amount.
    #[must_use]
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    /// Returns the raw gas amount.
    #[must_use]
    pub const fn get(self) -> u64 {
        self.0
    }

    /// Adds two gas amounts with overflow detection.
    pub const fn checked_add(self, other: Self) -> Result<Self, EvmCoreError> {
        match self.0.checked_add(other.0) {
            Some(value) => Ok(Self(value)),
            None => Err(EvmCoreError::GasOverflow),
        }
    }
}

/// Fork-scoped gas schedule for the currently executable opcode subset.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EvmGasSchedule {
    fork: EvmFork,
    gas_zero: EvmGas,
    gas_base: EvmGas,
    gas_very_low: EvmGas,
    gas_low: EvmGas,
    gas_mid: EvmGas,
    gas_high: EvmGas,
    gas_jumpdest: EvmGas,
    gas_memory: EvmGas,
    gas_selfbalance: EvmGas,
    gas_warm_access: EvmGas,
    gas_cold_account_access: EvmGas,
    gas_cold_sload: EvmGas,
    gas_copy: EvmGas,
}

impl EvmGasSchedule {
    /// Returns the schedule for a supported fork.
    pub const fn for_fork(fork: EvmFork) -> Result<Self, EvmCoreError> {
        if !fork.is_supported() {
            return Err(EvmCoreError::UnsupportedFork);
        }
        Ok(Self {
            fork,
            gas_zero: EvmGas::new(0),
            gas_base: EvmGas::new(2),
            gas_very_low: EvmGas::new(3),
            gas_low: EvmGas::new(5),
            gas_mid: EvmGas::new(8),
            gas_high: EvmGas::new(10),
            gas_jumpdest: EvmGas::new(1),
            gas_memory: EvmGas::new(3),
            gas_selfbalance: EvmGas::new(5),
            gas_warm_access: EvmGas::new(100),
            gas_cold_account_access: EvmGas::new(2_600),
            gas_cold_sload: EvmGas::new(2_100),
            gas_copy: EvmGas::new(3),
        })
    }

    /// Returns the fork this schedule was built for.
    #[must_use]
    pub const fn fork(self) -> EvmFork {
        self.fork
    }

    /// Returns whether this schedule admits warm/cold state-access gas.
    pub const fn require_warm_cold_state_access(self) -> Result<(), EvmCoreError> {
        if !self.fork.supports_warm_cold_state_access() {
            return Err(EvmCoreError::UnsupportedFork);
        }
        Ok(())
    }

    /// Returns the fixed base gas for an executable opcode.
    pub const fn base_cost(self, opcode: EvmOpcode) -> Result<EvmGas, EvmCoreError> {
        if !self.fork.opcode_is_introduced(opcode) {
            return Err(EvmCoreError::UnsupportedOpcode);
        }
        match opcode.byte() {
            0x00 | 0xf3 | 0xfd => Ok(self.gas_zero),
            0x02 => Ok(self.gas_low),
            0x50 | 0x58 => Ok(self.gas_base),
            0x56 => Ok(self.gas_mid),
            0x57 => Ok(self.gas_high),
            0x5b => Ok(self.gas_jumpdest),
            0x01 | 0x03 | 0x10 | 0x11 | 0x14..=0x19 | 0x60..=0x9f => Ok(self.gas_very_low),
            _ => Err(EvmCoreError::UnsupportedOpcode),
        }
    }

    /// Returns the gas for `SELFBALANCE`.
    #[must_use]
    pub const fn selfbalance_cost(self) -> EvmGas {
        self.gas_selfbalance
    }

    /// Returns the account access gas for warm or cold account reads.
    #[must_use]
    pub const fn account_access_cost(self, warm: bool) -> EvmGas {
        if warm {
            self.gas_warm_access
        } else {
            self.gas_cold_account_access
        }
    }

    /// Returns the storage access gas for warm or cold slot reads.
    #[must_use]
    pub const fn storage_access_cost(self, warm: bool) -> EvmGas {
        if warm {
            self.gas_warm_access
        } else {
            self.gas_cold_sload
        }
    }

    /// Returns copy gas for a byte length.
    pub fn copy_cost(self, len: usize) -> Result<EvmGas, EvmCoreError> {
        if len == 0 {
            return Ok(EvmGas::new(0));
        }
        let rounded = len
            .checked_add(EVM_COPY_GAS_WORD_BYTES - 1)
            .ok_or(EvmCoreError::GasOverflow)?;
        let words = rounded
            .checked_div(EVM_COPY_GAS_WORD_BYTES)
            .ok_or(EvmCoreError::GasOverflow)?;
        let words = u64::try_from(words).map_err(|_| EvmCoreError::GasOverflow)?;
        let cost = words
            .checked_mul(self.gas_copy.get())
            .ok_or(EvmCoreError::GasOverflow)?;
        Ok(EvmGas::new(cost))
    }

    /// Computes memory expansion cost for a new active memory word count.
    pub fn memory_cost(self, words: u64) -> Result<EvmGas, EvmCoreError> {
        let linear = words
            .checked_mul(self.gas_memory.get())
            .ok_or(EvmCoreError::GasOverflow)?;
        let square = words.checked_mul(words).ok_or(EvmCoreError::GasOverflow)?;
        let quadratic = square
            .checked_div(EVM_MEMORY_QUADRATIC_DIVISOR)
            .ok_or(EvmCoreError::GasOverflow)?;
        EvmGas::new(linear).checked_add(EvmGas::new(quadratic))
    }

    /// Computes the incremental memory expansion cost for a memory range.
    pub fn memory_expansion_cost(
        self,
        current_words: u64,
        offset: usize,
        len: usize,
    ) -> Result<(u64, EvmGas), EvmCoreError> {
        if len == 0 {
            return Ok((current_words, EvmGas::new(0)));
        }
        let end = offset.checked_add(len).ok_or(EvmCoreError::GasOverflow)?;
        let rounded = end
            .checked_add(EVM_MEMORY_WORD_BYTES - 1)
            .ok_or(EvmCoreError::GasOverflow)?;
        let new_words_usize = rounded
            .checked_div(EVM_MEMORY_WORD_BYTES)
            .ok_or(EvmCoreError::GasOverflow)?;
        let new_words = u64::try_from(new_words_usize).map_err(|_| EvmCoreError::GasOverflow)?;
        if new_words <= current_words {
            return Ok((current_words, EvmGas::new(0)));
        }
        let new_cost = self.memory_cost(new_words)?;
        let current_cost = self.memory_cost(current_words)?;
        let delta = new_cost
            .get()
            .checked_sub(current_cost.get())
            .ok_or(EvmCoreError::GasOverflow)?;
        Ok((new_words, EvmGas::new(delta)))
    }
}

/// Stateful gas meter for one deterministic execution attempt.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EvmGasMeter {
    limit: EvmGas,
    used: EvmGas,
    active_memory_words: u64,
}

impl EvmGasMeter {
    /// Creates a gas meter with an explicit gas limit.
    pub const fn try_new(limit: EvmGas) -> Result<Self, EvmCoreError> {
        if limit.get() == 0 {
            return Err(EvmCoreError::ExecutionGasLimitZero);
        }
        if limit.get() > EVM_MAX_GAS_LIMIT {
            return Err(EvmCoreError::ExecutionGasLimitTooLarge);
        }
        Ok(Self {
            limit,
            used: EvmGas::new(0),
            active_memory_words: 0,
        })
    }

    /// Returns the gas limit.
    #[must_use]
    pub const fn limit(self) -> EvmGas {
        self.limit
    }

    /// Returns the gas consumed so far.
    #[must_use]
    pub const fn used(self) -> EvmGas {
        self.used
    }

    /// Returns the remaining gas.
    pub const fn remaining(self) -> Result<EvmGas, EvmCoreError> {
        match self.limit.get().checked_sub(self.used.get()) {
            Some(value) => Ok(EvmGas::new(value)),
            None => Err(EvmCoreError::GasOverflow),
        }
    }

    /// Charges fixed gas before an opcode applies side effects.
    pub fn charge(&mut self, cost: EvmGas) -> Result<(), EvmCoreError> {
        let next = self.used.checked_add(cost)?;
        if next > self.limit {
            return Err(EvmCoreError::OutOfGas);
        }
        self.used = next;
        Ok(())
    }

    /// Charges memory expansion gas for a checked memory range.
    pub fn charge_memory_range(
        &mut self,
        schedule: EvmGasSchedule,
        offset: usize,
        len: usize,
    ) -> Result<(), EvmCoreError> {
        let (new_words, cost) =
            schedule.memory_expansion_cost(self.active_memory_words, offset, len)?;
        self.charge(cost)?;
        self.active_memory_words = new_words;
        Ok(())
    }
}
