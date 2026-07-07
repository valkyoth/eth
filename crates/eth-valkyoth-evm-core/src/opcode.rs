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
    /// `LT`.
    pub const LT: Self = Self(0x10);
    /// `GT`.
    pub const GT: Self = Self(0x11);
    /// `BALANCE`.
    pub const BALANCE: Self = Self(0x31);
    /// `EXTCODESIZE`.
    pub const EXTCODESIZE: Self = Self(0x3b);
    /// `EXTCODECOPY`.
    pub const EXTCODECOPY: Self = Self(0x3c);
    /// `EXTCODEHASH`.
    pub const EXTCODEHASH: Self = Self(0x3f);
    /// `SELFBALANCE`.
    pub const SELFBALANCE: Self = Self(0x47);
    /// `EQ`.
    pub const EQ: Self = Self(0x14);
    /// `ISZERO`.
    pub const ISZERO: Self = Self(0x15);
    /// `AND`.
    pub const AND: Self = Self(0x16);
    /// `OR`.
    pub const OR: Self = Self(0x17);
    /// `XOR`.
    pub const XOR: Self = Self(0x18);
    /// `NOT`.
    pub const NOT: Self = Self(0x19);
    /// `POP`.
    pub const POP: Self = Self(0x50);
    /// `SLOAD`.
    pub const SLOAD: Self = Self(0x54);
    /// `SSTORE`.
    pub const SSTORE: Self = Self(0x55);
    /// `CREATE`.
    pub const CREATE: Self = Self(0xf0);
    /// `CALL`.
    pub const CALL: Self = Self(0xf1);
    /// `CALLCODE`.
    pub const CALLCODE: Self = Self(0xf2);
    /// `DELEGATECALL`.
    pub const DELEGATECALL: Self = Self(0xf4);
    /// `CREATE2`.
    pub const CREATE2: Self = Self(0xf5);
    /// `STATICCALL`.
    pub const STATICCALL: Self = Self(0xfa);
    /// `MLOAD`.
    pub const MLOAD: Self = Self(0x51);
    /// `MSTORE`.
    pub const MSTORE: Self = Self(0x52);
    /// `JUMP`.
    pub const JUMP: Self = Self(0x56);
    /// `JUMPI`.
    pub const JUMPI: Self = Self(0x57);
    /// `PC`.
    pub const PC: Self = Self(0x58);
    /// `JUMPDEST`.
    pub const JUMPDEST: Self = Self(0x5b);
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
    /// `RETURN`.
    pub const RETURN: Self = Self(0xf3);
    /// `REVERT`.
    pub const REVERT: Self = Self(0xfd);

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

    /// Returns whether this opcode is in the `DUP1..=DUP16` range.
    #[must_use]
    pub const fn is_dup(self) -> bool {
        self.0 >= Self::DUP1.0 && self.0 <= Self::DUP16.0
    }

    /// Returns the zero-based stack depth for a `DUPn` opcode.
    #[must_use]
    pub const fn dup_depth(self) -> Option<u8> {
        if !self.is_dup() {
            return None;
        }
        self.0.checked_sub(Self::DUP1.0)
    }

    /// Returns whether this opcode is in the `SWAP1..=SWAP16` range.
    #[must_use]
    pub const fn is_swap(self) -> bool {
        self.0 >= Self::SWAP1.0 && self.0 <= Self::SWAP16.0
    }

    /// Returns the one-based stack depth for a `SWAPn` opcode.
    #[must_use]
    pub const fn swap_depth(self) -> Option<u8> {
        if !self.is_swap() {
            return None;
        }
        self.0.checked_sub(0x8f)
    }

    /// Returns whether this opcode requires explicit host state.
    #[must_use]
    pub const fn is_state_access(self) -> bool {
        matches!(self.0, 0x31 | 0x3b | 0x3c | 0x3f | 0x47 | 0x54 | 0x55)
    }

    /// Returns whether this opcode is in the call/create family.
    #[must_use]
    pub const fn is_call_create(self) -> bool {
        matches!(self.0, 0xf0..=0xf2 | 0xf4 | 0xf5 | 0xfa)
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
    /// Bitwise stack operation.
    Bitwise,
    /// Comparison stack operation.
    Comparison,
    /// Memory operation.
    Memory,
    /// State access operation.
    State,
    /// Control-flow operation.
    ControlFlow,
    /// Call/create operation.
    CallCreate,
}

/// Fork-aware opcode metadata.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct OpcodeInfo {
    /// Opcode byte.
    pub opcode: EvmOpcode,
    /// Coarse opcode category.
    pub class: OpcodeClass,
}
