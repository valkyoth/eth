/// Maximum number of EIP-712 struct types admitted by the bounded encoder.
pub const EIP712_MAX_TYPES: usize = 64;
/// Maximum number of fields admitted in one borrowed EIP-712 struct type.
pub const EIP712_MAX_FIELDS_PER_TYPE: usize = 64;
/// Maximum number of named values admitted in one borrowed EIP-712 struct.
pub const EIP712_MAX_VALUES_PER_STRUCT: usize = 64;
/// Maximum elements admitted at any borrowed EIP-712 array dimension.
pub const EIP712_MAX_ARRAY_ITEMS: usize = 256;
/// Maximum recursive EIP-712 value visits admitted in one operation.
pub const EIP712_MAX_VALUE_NODES: usize = 4096;
/// Maximum cumulative dynamic `bytes` and UTF-8 string bytes hashed per operation.
pub const EIP712_MAX_DYNAMIC_VALUE_BYTES: usize = 1_048_576;

/// Work limits for borrowed EIP-712 encoding and hashing.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Eip712Limits {
    /// Maximum recursive value visits, including array containers.
    pub max_value_nodes: usize,
    /// Maximum cumulative dynamic `bytes` and UTF-8 string bytes hashed.
    ///
    /// A complete signing-digest operation charges both domain name/version
    /// strings and all dynamic message values against this one budget.
    pub max_dynamic_value_bytes: usize,
}

impl Eip712Limits {
    /// Conservative default limits used by the compatibility entry points.
    pub const DEFAULT: Self = Self {
        max_value_nodes: EIP712_MAX_VALUE_NODES,
        max_dynamic_value_bytes: EIP712_MAX_DYNAMIC_VALUE_BYTES,
    };
}
