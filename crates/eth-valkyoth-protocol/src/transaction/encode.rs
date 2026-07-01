use eth_valkyoth_codec::{
    DecodeError, checked_len_add, encode_rlp_integer, encode_rlp_list_header, encode_rlp_scalar,
    encoded_rlp_integer_len, encoded_rlp_list_header_len, encoded_rlp_scalar_len,
};
use eth_valkyoth_primitives::Address;

use super::access_list::ACCESS_LIST_TRANSACTION_TYPE;
use super::blob::BLOB_TRANSACTION_TYPE;
use super::dynamic_fee::DYNAMIC_FEE_TRANSACTION_TYPE;
use super::{
    AccessListTransactionTo, BlobVersionedHashes, UnvalidatedAccessListTransaction,
    UnvalidatedBlobTransaction, UnvalidatedDynamicFeeTransaction, UnvalidatedLegacyTransaction,
};
use crate::transaction::LegacyTransactionTo;

mod error;
#[cfg(test)]
mod tests;

pub use error::{TransactionEncodeError, TransactionEncodeErrorCategory};

const U64_BYTES: usize = 8;
const U256_BYTES: usize = 32;

/// Supported unvalidated transaction domains accepted by the encoder.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UnvalidatedTransaction<'a> {
    /// Legacy transaction.
    Legacy(UnvalidatedLegacyTransaction<'a>),
    /// EIP-2930 access-list transaction.
    AccessList(UnvalidatedAccessListTransaction<'a>),
    /// EIP-1559 dynamic-fee transaction.
    DynamicFee(UnvalidatedDynamicFeeTransaction<'a>),
    /// EIP-4844 blob transaction.
    Blob(UnvalidatedBlobTransaction<'a>),
}

/// Returns the canonical encoded transaction envelope length.
pub fn encoded_transaction_len(
    transaction: UnvalidatedTransaction<'_>,
) -> Result<usize, TransactionEncodeError> {
    match transaction {
        UnvalidatedTransaction::Legacy(tx) => encoded_legacy_transaction_len(&tx),
        UnvalidatedTransaction::AccessList(tx) => encoded_access_list_transaction_len(&tx),
        UnvalidatedTransaction::DynamicFee(tx) => encoded_dynamic_fee_transaction_len(&tx),
        UnvalidatedTransaction::Blob(tx) => encoded_blob_transaction_len(&tx),
    }
}

/// Canonically encodes an admitted unvalidated transaction envelope.
///
/// Unknown typed transaction payloads are intentionally not accepted here:
/// callers that need lossless forwarding of unsupported types should carry the
/// original bytes instead of constructing a typed field model this crate cannot
/// validate or re-encode.
pub fn encode_transaction(
    transaction: UnvalidatedTransaction<'_>,
    output: &mut [u8],
) -> Result<usize, TransactionEncodeError> {
    match transaction {
        UnvalidatedTransaction::Legacy(tx) => encode_legacy_transaction(&tx, output),
        UnvalidatedTransaction::AccessList(tx) => encode_access_list_transaction(&tx, output),
        UnvalidatedTransaction::DynamicFee(tx) => encode_dynamic_fee_transaction(&tx, output),
        UnvalidatedTransaction::Blob(tx) => encode_blob_transaction(&tx, output),
    }
}

/// Returns the canonical encoded legacy transaction length.
pub fn encoded_legacy_transaction_len(
    transaction: &UnvalidatedLegacyTransaction<'_>,
) -> Result<usize, TransactionEncodeError> {
    encoded_list_envelope_len(legacy_payload_len(transaction)?).map_err(Into::into)
}

/// Canonically encodes a legacy transaction.
pub fn encode_legacy_transaction(
    transaction: &UnvalidatedLegacyTransaction<'_>,
    output: &mut [u8],
) -> Result<usize, TransactionEncodeError> {
    let payload_len = legacy_payload_len(transaction)?;
    encode_list_envelope(payload_len, output, |fields| {
        write_u64(transaction.nonce.get(), fields)?;
        write_wei(transaction.gas_price.to_be_bytes(), fields)?;
        write_u64(transaction.gas_limit.get(), fields)?;
        write_legacy_to(transaction.to, fields)?;
        write_wei(transaction.value.to_be_bytes(), fields)?;
        write_scalar(transaction.input, fields)?;
        write_u256(transaction.v, fields)?;
        write_u256(transaction.r, fields)?;
        write_u256(transaction.s, fields)
    })
    .map_err(Into::into)
}

/// Returns the canonical encoded EIP-2930 transaction length.
pub fn encoded_access_list_transaction_len(
    transaction: &UnvalidatedAccessListTransaction<'_>,
) -> Result<usize, TransactionEncodeError> {
    encoded_typed_envelope_len(access_list_payload_len(transaction)?).map_err(Into::into)
}

/// Canonically encodes an EIP-2930 access-list transaction.
pub fn encode_access_list_transaction(
    transaction: &UnvalidatedAccessListTransaction<'_>,
    output: &mut [u8],
) -> Result<usize, TransactionEncodeError> {
    let payload_len = access_list_payload_len(transaction)?;
    encode_typed_envelope(
        ACCESS_LIST_TRANSACTION_TYPE,
        payload_len,
        output,
        |fields| {
            write_u64(transaction.chain_id.get(), fields)?;
            write_u64(transaction.nonce.get(), fields)?;
            write_wei(transaction.gas_price.to_be_bytes(), fields)?;
            write_u64(transaction.gas_limit.get(), fields)?;
            write_transaction_to(transaction.to, fields)?;
            write_wei(transaction.value.to_be_bytes(), fields)?;
            write_scalar(transaction.input, fields)?;
            write_access_list(transaction.access_list, fields)?;
            write_u64(u64::from(transaction.y_parity.get()), fields)?;
            write_u256(transaction.r, fields)?;
            write_u256(transaction.s, fields)
        },
    )
    .map_err(Into::into)
}

/// Returns the canonical encoded EIP-1559 transaction length.
pub fn encoded_dynamic_fee_transaction_len(
    transaction: &UnvalidatedDynamicFeeTransaction<'_>,
) -> Result<usize, TransactionEncodeError> {
    encoded_typed_envelope_len(dynamic_fee_payload_len(transaction)?).map_err(Into::into)
}

/// Canonically encodes an EIP-1559 dynamic-fee transaction.
pub fn encode_dynamic_fee_transaction(
    transaction: &UnvalidatedDynamicFeeTransaction<'_>,
    output: &mut [u8],
) -> Result<usize, TransactionEncodeError> {
    let payload_len = dynamic_fee_payload_len(transaction)?;
    encode_typed_envelope(
        DYNAMIC_FEE_TRANSACTION_TYPE,
        payload_len,
        output,
        |fields| {
            write_u64(transaction.chain_id.get(), fields)?;
            write_u64(transaction.nonce.get(), fields)?;
            write_wei(transaction.max_priority_fee_per_gas.to_be_bytes(), fields)?;
            write_wei(transaction.max_fee_per_gas.to_be_bytes(), fields)?;
            write_u64(transaction.gas_limit.get(), fields)?;
            write_transaction_to(transaction.to, fields)?;
            write_wei(transaction.value.to_be_bytes(), fields)?;
            write_scalar(transaction.input, fields)?;
            write_access_list(transaction.access_list, fields)?;
            write_u64(u64::from(transaction.y_parity.get()), fields)?;
            write_u256(transaction.r, fields)?;
            write_u256(transaction.s, fields)
        },
    )
    .map_err(Into::into)
}

/// Returns the canonical encoded EIP-4844 transaction length.
pub fn encoded_blob_transaction_len(
    transaction: &UnvalidatedBlobTransaction<'_>,
) -> Result<usize, TransactionEncodeError> {
    encoded_typed_envelope_len(blob_payload_len(transaction)?).map_err(Into::into)
}

/// Canonically encodes an EIP-4844 blob transaction.
pub fn encode_blob_transaction(
    transaction: &UnvalidatedBlobTransaction<'_>,
    output: &mut [u8],
) -> Result<usize, TransactionEncodeError> {
    let payload_len = blob_payload_len(transaction)?;
    encode_typed_envelope(BLOB_TRANSACTION_TYPE, payload_len, output, |fields| {
        write_u64(transaction.chain_id.get(), fields)?;
        write_u64(transaction.nonce.get(), fields)?;
        write_wei(transaction.max_priority_fee_per_gas.to_be_bytes(), fields)?;
        write_wei(transaction.max_fee_per_gas.to_be_bytes(), fields)?;
        write_u64(transaction.gas_limit.get(), fields)?;
        write_address(transaction.to, fields)?;
        write_wei(transaction.value.to_be_bytes(), fields)?;
        write_scalar(transaction.input, fields)?;
        write_access_list(transaction.access_list, fields)?;
        write_wei(transaction.max_fee_per_blob_gas.to_be_bytes(), fields)?;
        write_blob_hashes(transaction.blob_versioned_hashes, fields)?;
        write_u64(u64::from(transaction.y_parity.get()), fields)?;
        write_u256(transaction.r, fields)?;
        write_u256(transaction.s, fields)
    })
    .map_err(Into::into)
}

fn legacy_payload_len(tx: &UnvalidatedLegacyTransaction<'_>) -> Result<usize, DecodeError> {
    sum_lengths(&[
        encoded_u64_len(tx.nonce.get())?,
        encoded_u256_len(tx.gas_price.to_be_bytes())?,
        encoded_u64_len(tx.gas_limit.get())?,
        encoded_legacy_to_len(tx.to)?,
        encoded_u256_len(tx.value.to_be_bytes())?,
        encoded_rlp_scalar_len(tx.input)?,
        encoded_u256_len(tx.v)?,
        encoded_u256_len(tx.r)?,
        encoded_u256_len(tx.s)?,
    ])
}

fn access_list_payload_len(
    tx: &UnvalidatedAccessListTransaction<'_>,
) -> Result<usize, DecodeError> {
    sum_lengths(&[
        encoded_u64_len(tx.chain_id.get())?,
        encoded_u64_len(tx.nonce.get())?,
        encoded_u256_len(tx.gas_price.to_be_bytes())?,
        encoded_u64_len(tx.gas_limit.get())?,
        encoded_access_list_to_len(tx.to)?,
        encoded_u256_len(tx.value.to_be_bytes())?,
        encoded_rlp_scalar_len(tx.input)?,
        tx.access_list.encoded_rlp_len(),
        encoded_u64_len(u64::from(tx.y_parity.get()))?,
        encoded_u256_len(tx.r)?,
        encoded_u256_len(tx.s)?,
    ])
}

fn dynamic_fee_payload_len(
    tx: &UnvalidatedDynamicFeeTransaction<'_>,
) -> Result<usize, DecodeError> {
    sum_lengths(&[
        encoded_u64_len(tx.chain_id.get())?,
        encoded_u64_len(tx.nonce.get())?,
        encoded_u256_len(tx.max_priority_fee_per_gas.to_be_bytes())?,
        encoded_u256_len(tx.max_fee_per_gas.to_be_bytes())?,
        encoded_u64_len(tx.gas_limit.get())?,
        encoded_access_list_to_len(tx.to)?,
        encoded_u256_len(tx.value.to_be_bytes())?,
        encoded_rlp_scalar_len(tx.input)?,
        tx.access_list.encoded_rlp_len(),
        encoded_u64_len(u64::from(tx.y_parity.get()))?,
        encoded_u256_len(tx.r)?,
        encoded_u256_len(tx.s)?,
    ])
}

fn blob_payload_len(tx: &UnvalidatedBlobTransaction<'_>) -> Result<usize, DecodeError> {
    sum_lengths(&[
        encoded_u64_len(tx.chain_id.get())?,
        encoded_u64_len(tx.nonce.get())?,
        encoded_u256_len(tx.max_priority_fee_per_gas.to_be_bytes())?,
        encoded_u256_len(tx.max_fee_per_gas.to_be_bytes())?,
        encoded_u64_len(tx.gas_limit.get())?,
        encoded_rlp_scalar_len(&tx.to.to_bytes())?,
        encoded_u256_len(tx.value.to_be_bytes())?,
        encoded_rlp_scalar_len(tx.input)?,
        tx.access_list.encoded_rlp_len(),
        encoded_u256_len(tx.max_fee_per_blob_gas.to_be_bytes())?,
        tx.blob_versioned_hashes.encoded_rlp_len(),
        encoded_u64_len(u64::from(tx.y_parity.get()))?,
        encoded_u256_len(tx.r)?,
        encoded_u256_len(tx.s)?,
    ])
}

fn encoded_list_envelope_len(payload_len: usize) -> Result<usize, DecodeError> {
    checked_len_add(encoded_rlp_list_header_len(payload_len)?, payload_len)
}

fn encoded_typed_envelope_len(payload_len: usize) -> Result<usize, DecodeError> {
    checked_len_add(1, encoded_list_envelope_len(payload_len)?)
}

fn encode_list_envelope(
    payload_len: usize,
    output: &mut [u8],
    write_fields: impl FnOnce(&mut FieldWriter<'_>) -> Result<(), DecodeError>,
) -> Result<usize, DecodeError> {
    let total_len = encoded_list_envelope_len(payload_len)?;
    if output.len() < total_len {
        return Err(DecodeError::OffsetOutOfBounds);
    }
    let header_len = encoded_rlp_list_header_len(payload_len)?;
    let fields = output
        .get_mut(header_len..total_len)
        .ok_or(DecodeError::OffsetOutOfBounds)?;
    let mut writer = FieldWriter::new(fields);
    write_fields(&mut writer)?;
    writer.finish(payload_len)?;
    encode_rlp_list_header(payload_len, output)?;
    Ok(total_len)
}

fn encode_typed_envelope(
    transaction_type: u8,
    payload_len: usize,
    output: &mut [u8],
    write_fields: impl FnOnce(&mut FieldWriter<'_>) -> Result<(), DecodeError>,
) -> Result<usize, DecodeError> {
    let payload_total_len = encoded_list_envelope_len(payload_len)?;
    let total_len = checked_len_add(1, payload_total_len)?;
    if output.len() < total_len {
        return Err(DecodeError::OffsetOutOfBounds);
    }
    let payload_output = output
        .get_mut(1..total_len)
        .ok_or(DecodeError::OffsetOutOfBounds)?;
    encode_list_envelope(payload_len, payload_output, write_fields)?;
    let type_byte = output.first_mut().ok_or(DecodeError::OffsetOutOfBounds)?;
    *type_byte = transaction_type;
    Ok(total_len)
}

fn encoded_u64_len(value: u64) -> Result<usize, DecodeError> {
    let bytes = value.to_be_bytes();
    encoded_rlp_integer_len(trim_u64_payload(&bytes))
}

fn encoded_u256_len(bytes: [u8; U256_BYTES]) -> Result<usize, DecodeError> {
    encoded_rlp_integer_len(trim_u256_payload(&bytes))
}

fn encoded_legacy_to_len(to: LegacyTransactionTo) -> Result<usize, DecodeError> {
    match to {
        LegacyTransactionTo::Create => encoded_rlp_scalar_len(&[]),
        LegacyTransactionTo::Call(address) => encoded_rlp_scalar_len(&address.to_bytes()),
    }
}

fn encoded_access_list_to_len(to: AccessListTransactionTo) -> Result<usize, DecodeError> {
    match to {
        AccessListTransactionTo::Create => encoded_rlp_scalar_len(&[]),
        AccessListTransactionTo::Call(address) => encoded_rlp_scalar_len(&address.to_bytes()),
    }
}

fn sum_lengths(lengths: &[usize]) -> Result<usize, DecodeError> {
    let mut total = 0usize;
    for length in lengths {
        total = checked_len_add(total, *length)?;
    }
    Ok(total)
}

struct FieldWriter<'a> {
    output: &'a mut [u8],
    cursor: usize,
}

impl<'a> FieldWriter<'a> {
    fn new(output: &'a mut [u8]) -> Self {
        Self { output, cursor: 0 }
    }

    fn write_with(
        &mut self,
        write: impl FnOnce(&mut [u8]) -> Result<usize, DecodeError>,
    ) -> Result<(), DecodeError> {
        let target = self
            .output
            .get_mut(self.cursor..)
            .ok_or(DecodeError::OffsetOutOfBounds)?;
        let written = write(target)?;
        self.cursor = checked_len_add(self.cursor, written)?;
        Ok(())
    }

    fn finish(self, expected_len: usize) -> Result<(), DecodeError> {
        if self.cursor == expected_len {
            Ok(())
        } else {
            Err(DecodeError::DecoderOverread)
        }
    }
}

fn write_u64(value: u64, writer: &mut FieldWriter<'_>) -> Result<(), DecodeError> {
    let bytes = value.to_be_bytes();
    writer.write_with(|output| encode_rlp_integer(trim_u64_payload(&bytes), output))
}

fn write_u256(bytes: [u8; U256_BYTES], writer: &mut FieldWriter<'_>) -> Result<(), DecodeError> {
    writer.write_with(|output| encode_rlp_integer(trim_u256_payload(&bytes), output))
}

fn write_wei(bytes: [u8; U256_BYTES], writer: &mut FieldWriter<'_>) -> Result<(), DecodeError> {
    write_u256(bytes, writer)
}

fn write_scalar(payload: &[u8], writer: &mut FieldWriter<'_>) -> Result<(), DecodeError> {
    writer.write_with(|output| encode_rlp_scalar(payload, output))
}

fn write_address(address: Address, writer: &mut FieldWriter<'_>) -> Result<(), DecodeError> {
    write_scalar(&address.to_bytes(), writer)
}

fn write_legacy_to(
    to: LegacyTransactionTo,
    writer: &mut FieldWriter<'_>,
) -> Result<(), DecodeError> {
    match to {
        LegacyTransactionTo::Create => write_scalar(&[], writer),
        LegacyTransactionTo::Call(address) => write_address(address, writer),
    }
}

fn write_transaction_to(
    to: AccessListTransactionTo,
    writer: &mut FieldWriter<'_>,
) -> Result<(), DecodeError> {
    match to {
        AccessListTransactionTo::Create => write_scalar(&[], writer),
        AccessListTransactionTo::Call(address) => write_address(address, writer),
    }
}

fn write_access_list(
    access_list: super::AccessList<'_>,
    writer: &mut FieldWriter<'_>,
) -> Result<(), DecodeError> {
    writer.write_with(|output| access_list.encode_rlp(output))
}

fn write_blob_hashes(
    hashes: BlobVersionedHashes<'_>,
    writer: &mut FieldWriter<'_>,
) -> Result<(), DecodeError> {
    writer.write_with(|output| hashes.encode_rlp(output))
}

fn trim_u64_payload(bytes: &[u8; U64_BYTES]) -> &[u8] {
    let start = bytes
        .iter()
        .position(|byte| *byte != 0)
        .unwrap_or(U64_BYTES);
    bytes.get(start..).unwrap_or(&[])
}

fn trim_u256_payload(bytes: &[u8; U256_BYTES]) -> &[u8] {
    let start = bytes
        .iter()
        .position(|byte| *byte != 0)
        .unwrap_or(U256_BYTES);
    bytes.get(start..).unwrap_or(&[])
}
