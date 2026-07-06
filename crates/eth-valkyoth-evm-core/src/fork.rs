use crate::{EvmCoreError, EvmOpcode, OpcodeClass, OpcodeInfo};

/// Numeric fork identifier for the native EVM skeleton.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct EvmFork(u16);

impl EvmFork {
    /// Frontier fork identifier.
    pub const FRONTIER: Self = Self(0);
    /// London fork identifier.
    pub const LONDON: Self = Self(1);
    /// Shanghai fork identifier.
    pub const SHANGHAI: Self = Self(2);
    /// Cancun fork identifier.
    pub const CANCUN: Self = Self(3);
    /// Prague fork identifier.
    pub const PRAGUE: Self = Self(4);

    /// Constructs a fork identifier.
    #[must_use]
    pub const fn new(identifier: u16) -> Self {
        Self(identifier)
    }

    /// Returns the numeric fork identifier.
    #[must_use]
    pub const fn get(self) -> u16 {
        self.0
    }

    /// Returns whether this fork is supported by the current skeleton table.
    #[must_use]
    pub const fn is_supported(self) -> bool {
        self.0 <= Self::PRAGUE.0
    }
}

/// Fork-aware opcode table skeleton.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct OpcodeTable {
    fork: EvmFork,
}

impl OpcodeTable {
    /// Creates an opcode table for a supported fork.
    pub const fn try_new(fork: EvmFork) -> Result<Self, EvmCoreError> {
        if !fork.is_supported() {
            return Err(EvmCoreError::UnsupportedFork);
        }
        Ok(Self { fork })
    }

    /// Returns the fork this table was created for.
    #[must_use]
    pub const fn fork(self) -> EvmFork {
        self.fork
    }

    /// Looks up coarse opcode metadata for the current fork skeleton.
    ///
    /// The table may classify opcode domains before the bootstrap interpreter
    /// executes them. Use [`crate::EvmExecution`] for the authoritative
    /// currently executable subset; unsupported dispatcher arms still fail
    /// closed with [`EvmCoreError::UnsupportedOpcode`].
    ///
    /// # Invariants
    ///
    /// `OpcodeTable` is constructed only through [`Self::try_new`], which
    /// rejects unsupported fork identifiers. Future mutators must preserve that
    /// invariant instead of relying on a second check here.
    pub const fn instruction(self, opcode: EvmOpcode) -> Result<OpcodeInfo, EvmCoreError> {
        let class = match opcode.byte() {
            0x00 => OpcodeClass::Stop,
            0x01..=0x03 => OpcodeClass::Arithmetic,
            0x10 | 0x11 | 0x14 | 0x15 => OpcodeClass::Comparison,
            0x16..=0x19 => OpcodeClass::Bitwise,
            0x31 | 0x3b | 0x3c | 0x3f | 0x47 | 0x54 | 0x55 => {
                if self.fork.get() < EvmFork::LONDON.get() {
                    return Err(EvmCoreError::UnsupportedOpcode);
                }
                OpcodeClass::State
            }
            0x50 | 0x58 | 0x5b | 0x60..=0x9f => OpcodeClass::Stack,
            0x51 | 0x52 => OpcodeClass::Memory,
            0x56 | 0x57 | 0xf3 | 0xfd => OpcodeClass::ControlFlow,
            _ => return Err(EvmCoreError::UnsupportedOpcode),
        };
        Ok(OpcodeInfo { opcode, class })
    }
}
