/// Raw EVM opcode byte.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct EvmOpcode(u8);

impl EvmOpcode {
    /// `STOP`.
    pub const STOP: Self = Self(0x00);
    /// `ADD`.
    pub const ADD: Self = Self(0x01);
    /// `MUL`.
    pub const MUL: Self = Self(0x02);
    /// `SUB`.
    pub const SUB: Self = Self(0x03);
    /// `POP`.
    pub const POP: Self = Self(0x50);
    /// `MLOAD`.
    pub const MLOAD: Self = Self(0x51);
    /// `MSTORE`.
    pub const MSTORE: Self = Self(0x52);
    /// `JUMP`.
    pub const JUMP: Self = Self(0x56);
    /// `JUMPI`.
    pub const JUMPI: Self = Self(0x57);
    /// First `PUSHn` opcode.
    pub const PUSH1: Self = Self(0x60);
    /// Last `PUSHn` opcode.
    pub const PUSH32: Self = Self(0x7f);
    /// First `DUPn` opcode.
    pub const DUP1: Self = Self(0x80);
    /// Last `DUPn` opcode.
    pub const DUP16: Self = Self(0x8f);
    /// First `SWAPn` opcode.
    pub const SWAP1: Self = Self(0x90);
    /// Last `SWAPn` opcode.
    pub const SWAP16: Self = Self(0x9f);

    /// Constructs an opcode domain from a raw byte.
    #[must_use]
    pub const fn new(byte: u8) -> Self {
        Self(byte)
    }

    /// Returns the raw opcode byte.
    #[must_use]
    pub const fn byte(self) -> u8 {
        self.0
    }

    /// Returns whether this opcode is in the `PUSH1..=PUSH32` range.
    #[must_use]
    pub const fn is_push(self) -> bool {
        self.0 >= Self::PUSH1.0 && self.0 <= Self::PUSH32.0
    }

    /// Returns the immediate width for a `PUSHn` opcode.
    #[must_use]
    pub const fn push_width(self) -> Option<u8> {
        if !self.is_push() {
            return None;
        }
        self.0.checked_sub(0x5f)
    }
}

/// Coarse opcode category used by the skeleton table.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OpcodeClass {
    /// Halts execution.
    Stop,
    /// Arithmetic stack operation.
    Arithmetic,
    /// Stack-only operation.
    Stack,
    /// Memory operation.
    Memory,
    /// Control-flow operation.
    ControlFlow,
}

/// Fork-aware opcode metadata.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct OpcodeInfo {
    /// Opcode byte.
    pub opcode: EvmOpcode,
    /// Coarse opcode category.
    pub class: OpcodeClass,
}
