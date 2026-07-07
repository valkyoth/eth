use crate::{EvmCoreError, EvmOpcode, OpcodeClass, OpcodeInfo};

/// Numeric fork identifier for the native EVM skeleton.
///
/// These are crate-local chronological identifiers used for table ordering.
/// They are not Ethereum consensus, network, or wire identifiers.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct EvmFork(u16);

impl EvmFork {
    /// Frontier fork identifier.
    pub const FRONTIER: Self = Self(0);
    /// Homestead fork identifier.
    pub const HOMESTEAD: Self = Self(1);
    /// Tangerine Whistle fork identifier.
    pub const TANGERINE_WHISTLE: Self = Self(2);
    /// Spurious Dragon fork identifier.
    pub const SPURIOUS_DRAGON: Self = Self(3);
    /// Byzantium fork identifier.
    pub const BYZANTIUM: Self = Self(4);
    /// Constantinople fork identifier.
    pub const CONSTANTINOPLE: Self = Self(5);
    /// Petersburg fork identifier.
    pub const PETERSBURG: Self = Self(6);
    /// Istanbul fork identifier.
    pub const ISTANBUL: Self = Self(7);
    /// Berlin fork identifier.
    pub const BERLIN: Self = Self(8);
    /// London fork identifier.
    pub const LONDON: Self = Self(9);
    /// Shanghai fork identifier.
    pub const SHANGHAI: Self = Self(10);
    /// Cancun fork identifier.
    pub const CANCUN: Self = Self(11);
    /// Prague fork identifier.
    pub const PRAGUE: Self = Self(12);
    /// Amsterdam fork identifier reserved for future fork planning.
    pub const AMSTERDAM: Self = Self(13);

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

    /// Returns whether this fork is known to the roadmap, even if unsupported.
    #[must_use]
    pub const fn is_known(self) -> bool {
        self.0 <= Self::AMSTERDAM.0
    }

    /// Returns whether warm/cold state gas is claimed for this fork.
    #[must_use]
    pub const fn supports_warm_cold_state_access(self) -> bool {
        self.0 >= Self::BERLIN.0 && self.is_supported()
    }

    /// Returns the first fork where the modeled opcode exists.
    #[must_use]
    pub const fn opcode_introduced_in(opcode: EvmOpcode) -> Option<Self> {
        match opcode.byte() {
            0x00
            | 0x01..=0x03
            | 0x10
            | 0x11
            | 0x14..=0x19
            | 0x31
            | 0x3b
            | 0x3c
            | 0x50
            | 0x51
            | 0x52
            | 0x54..=0x58
            | 0x5b
            | 0x60..=0x9f
            | 0xf0..=0xf2
            | 0xf3 => Some(Self::FRONTIER),
            0xf4 => Some(Self::HOMESTEAD),
            0xf5 => Some(Self::CONSTANTINOPLE),
            0xfd => Some(Self::BYZANTIUM),
            0xfa => Some(Self::BYZANTIUM),
            0x3f => Some(Self::CONSTANTINOPLE),
            0x47 => Some(Self::ISTANBUL),
            _ => None,
        }
    }

    /// Returns whether the modeled opcode exists in this fork.
    #[must_use]
    pub const fn opcode_is_introduced(self, opcode: EvmOpcode) -> bool {
        match Self::opcode_introduced_in(opcode) {
            Some(introduced_in) => self.0 >= introduced_in.0,
            None => false,
        }
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
        if !self.fork.opcode_is_introduced(opcode) {
            return Err(EvmCoreError::UnsupportedOpcode);
        }
        let class = match opcode.byte() {
            0x00 => OpcodeClass::Stop,
            0x01..=0x03 => OpcodeClass::Arithmetic,
            0x10 | 0x11 | 0x14 | 0x15 => OpcodeClass::Comparison,
            0x16..=0x19 => OpcodeClass::Bitwise,
            0x31 | 0x3b | 0x3c | 0x3f | 0x47 | 0x54 | 0x55 => OpcodeClass::State,
            0x50 | 0x58 | 0x5b | 0x60..=0x9f => OpcodeClass::Stack,
            0x51 | 0x52 => OpcodeClass::Memory,
            0x56 | 0x57 | 0xf3 | 0xfd => OpcodeClass::ControlFlow,
            0xf0..=0xf2 | 0xf4 | 0xf5 | 0xfa => OpcodeClass::CallCreate,
            _ => return Err(EvmCoreError::UnsupportedOpcode),
        };
        Ok(OpcodeInfo { opcode, class })
    }
}
