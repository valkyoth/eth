use eth_valkyoth_hash::{Keccak256, hash_one};
use eth_valkyoth_primitives::B256;

use super::typed_helpers::find_struct;
use super::{
    EIP712_MAX_TYPES, EIP712_MAX_VALUE_NODES, Eip712EncodeError, Eip712StructType,
    encode_eip712_type_validated, type_graph::validate_schema,
};

pub(super) struct ValidatedSchema<'a> {
    types: &'a [Eip712StructType<'a>],
    type_hashes: [Option<B256>; EIP712_MAX_TYPES],
    remaining_value_nodes: usize,
}

impl<'a> ValidatedSchema<'a> {
    pub(super) fn try_new(types: &'a [Eip712StructType<'a>]) -> Result<Self, Eip712EncodeError> {
        validate_schema(types)?;
        Ok(Self {
            types,
            type_hashes: [None; EIP712_MAX_TYPES],
            remaining_value_nodes: EIP712_MAX_VALUE_NODES,
        })
    }

    pub(super) const fn types(&self) -> &'a [Eip712StructType<'a>] {
        self.types
    }

    pub(super) fn contains_struct(&self, name: &str) -> bool {
        self.types.iter().any(|ty| ty.name == name)
    }

    pub(super) fn charge_value_node(&mut self) -> Result<(), Eip712EncodeError> {
        self.remaining_value_nodes = self
            .remaining_value_nodes
            .checked_sub(1)
            .ok_or(Eip712EncodeError::ResourceLimit)?;
        Ok(())
    }

    pub(super) fn type_hash<H>(
        &mut self,
        primary_type: &str,
        scratch: &mut [u8],
    ) -> Result<B256, Eip712EncodeError>
    where
        H: Default + Keccak256,
    {
        let primary = find_struct(self.types, primary_type)?;
        let index = self
            .types
            .iter()
            .position(|ty| ty.name == primary.name)
            .ok_or(Eip712EncodeError::UnknownStruct)?;
        if let Some(hash) = self
            .type_hashes
            .get(index)
            .copied()
            .ok_or(Eip712EncodeError::SchemaTooLarge)?
        {
            return Ok(hash);
        }
        let len = encode_eip712_type_validated(self.types, primary_type, scratch)?;
        let encoded = scratch
            .get(..len)
            .ok_or(Eip712EncodeError::OutputTooShort)?;
        let hash = hash_one(H::default(), encoded);
        let slot = self
            .type_hashes
            .get_mut(index)
            .ok_or(Eip712EncodeError::SchemaTooLarge)?;
        *slot = Some(hash);
        Ok(hash)
    }
}
