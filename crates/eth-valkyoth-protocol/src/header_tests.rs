use eth_valkyoth_codec::DecodeLimits;
use eth_valkyoth_hash::{Keccak256, Keccak256Digest};
use eth_valkyoth_primitives::B256;
extern crate std;
use std::{vec, vec::Vec};

use super::*;

const TEST_LIMITS: DecodeLimits = DecodeLimits {
    max_input_bytes: 1024,
    max_list_items: 32,
    max_nesting_depth: 4,
    max_total_allocation: 1024,
    max_proof_nodes: 4,
    max_total_items: 64,
};

#[test]
fn decodes_legacy_header_without_optional_fork_fields() -> Result<(), &'static str> {
    let input = header_rlp(HeaderFieldSet::Legacy);
    let header = decode_ok(&input, HeaderFieldSet::Legacy)?;

    assert_eq!(header.field_set, HeaderFieldSet::Legacy);
    assert_eq!(header.parent_hash, B256::from_bytes(bytes32(1)));
    assert_eq!(header.beneficiary.to_bytes(), bytes20(3));
    assert_eq!(header.logs_bloom.to_bytes(), bloom_bytes());
    assert_eq!(header.number.get(), 1);
    assert_eq!(header.gas_limit.get(), 21_000);
    assert_eq!(header.gas_used.get(), 20_000);
    assert_eq!(header.timestamp.get(), 12);
    assert_eq!(header.extra_data, b"eth");
    assert_eq!(header.nonce, [0x42; 8]);
    assert_eq!(header.base_fee_per_gas, None);
    assert_eq!(header.withdrawals_root, None);
    assert_eq!(header.blob_gas_used, None);
    assert_eq!(header.excess_blob_gas, None);
    assert_eq!(header.parent_beacon_block_root, None);
    assert_eq!(header.requests_hash, None);
    Ok(())
}

#[test]
fn decodes_prague_header_with_all_current_optional_fields() -> Result<(), &'static str> {
    let input = header_rlp(HeaderFieldSet::Prague);
    let header = decode_ok(&input, HeaderFieldSet::Prague)?;

    assert_eq!(
        header.base_fee_per_gas.map(|fee| fee.to_be_bytes()[31]),
        Some(7)
    );
    assert_eq!(header.withdrawals_root, Some(B256::from_bytes(bytes32(16))));
    assert_eq!(header.blob_gas_used.map(|gas| gas.get()), Some(3));
    assert_eq!(header.excess_blob_gas.map(|gas| gas.get()), Some(4));
    assert_eq!(
        header.parent_beacon_block_root,
        Some(B256::from_bytes(bytes32(19)))
    );
    assert_eq!(header.requests_hash, Some(B256::from_bytes(bytes32(20))));
    Ok(())
}

#[test]
fn rejects_header_field_count_for_selected_fork() -> Result<(), &'static str> {
    let input = header_rlp(HeaderFieldSet::Legacy);
    let error = decode_err(&input, HeaderFieldSet::London)?;

    assert_eq!(
        error,
        BlockHeaderDecodeError::WrongFieldCount {
            expected: LONDON_HEADER_FIELD_COUNT,
            found: LEGACY_HEADER_FIELD_COUNT
        }
    );
    assert_eq!(error.code(), "ETH_HEADER_WRONG_FIELD_COUNT");
    assert_eq!(
        error.category(),
        BlockHeaderDecodeErrorCategory::MalformedInput
    );
    Ok(())
}

#[test]
fn rejects_fixed_width_field_with_wrong_length() -> Result<(), &'static str> {
    let mut fields = base_fields();
    let Some(parent_hash) = fields.get_mut(0) else {
        return Err("base header fixture must include parent hash");
    };
    *parent_hash = scalar(&bytes31(1));
    let input = list(&fields);
    let error = decode_err(&input, HeaderFieldSet::Legacy)?;

    assert_eq!(
        error,
        BlockHeaderDecodeError::InvalidFieldLength {
            field: BlockHeaderField::ParentHash,
            expected: 32,
            found: 31
        }
    );
    Ok(())
}

#[test]
fn header_hash_uses_exact_canonical_rlp_bytes() -> Result<(), &'static str> {
    let input = header_rlp(HeaderFieldSet::Shanghai);
    let header = decode_ok(&input, HeaderFieldSet::Shanghai)?;
    let hash = header.hash_with(TranscriptHasher::default()).to_b256();
    let digest = hash.to_bytes();
    let Some(first) = input.first() else {
        return Err("header fixture must not be empty");
    };
    let Some(last) = input.last() else {
        return Err("header fixture must not be empty");
    };

    assert_eq!(header.encoded_rlp(), input.as_slice());
    assert_eq!(digest[0], usize_to_u8(input.len()));
    assert_eq!(digest[1], *first);
    assert_eq!(digest[2], *last);
    assert_eq!(digest[3], 1);
    Ok(())
}

#[derive(Default)]
struct TranscriptHasher {
    digest: [u8; 32],
    calls: u8,
}

impl Keccak256 for TranscriptHasher {
    fn update(&mut self, input: &[u8]) {
        self.calls = self.calls.saturating_add(1);
        self.digest[0] = usize_to_u8(input.len());
        if let Some(first) = input.first() {
            self.digest[1] = *first;
        }
        if let Some(last) = input.last() {
            self.digest[2] = *last;
        }
        self.digest[3] = self.calls;
    }

    fn finalize(self) -> Keccak256Digest {
        B256::from_bytes(self.digest)
    }
}

fn header_rlp(field_set: HeaderFieldSet) -> Vec<u8> {
    let mut fields = base_fields();
    if field_set.has_base_fee() {
        fields.push(uint(7));
    }
    if field_set.has_withdrawals_root() {
        fields.push(scalar(&bytes32(16)));
    }
    if field_set.has_cancun_fields() {
        fields.push(uint(3));
        fields.push(uint(4));
        fields.push(scalar(&bytes32(19)));
    }
    if field_set.has_requests_hash() {
        fields.push(scalar(&bytes32(20)));
    }
    list(&fields)
}

fn base_fields() -> Vec<Vec<u8>> {
    vec![
        scalar(&bytes32(1)),
        scalar(&bytes32(2)),
        scalar(&bytes20(3)),
        scalar(&bytes32(4)),
        scalar(&bytes32(5)),
        scalar(&bytes32(6)),
        scalar(&bloom_bytes()),
        uint(0),
        uint(1),
        uint(21_000),
        uint(20_000),
        uint(12),
        scalar(b"eth"),
        scalar(&bytes32(14)),
        scalar(&[0x42; 8]),
    ]
}

fn bytes20(seed: u8) -> [u8; 20] {
    core::array::from_fn(|index| seed.wrapping_add(usize_to_u8(index)))
}

fn bytes31(seed: u8) -> [u8; 31] {
    core::array::from_fn(|index| seed.wrapping_add(usize_to_u8(index)))
}

fn bytes32(seed: u8) -> [u8; 32] {
    core::array::from_fn(|index| seed.wrapping_add(usize_to_u8(index)))
}

fn bloom_bytes() -> [u8; 256] {
    core::array::from_fn(usize_to_u8)
}

fn uint(value: u64) -> Vec<u8> {
    if value == 0 {
        return scalar(&[]);
    }
    let bytes = value.to_be_bytes();
    let Some(first) = bytes.iter().position(|byte| *byte != 0) else {
        return scalar(&[]);
    };
    let Some(payload) = bytes.get(first..) else {
        return scalar(&[]);
    };
    scalar(payload)
}

fn scalar(payload: &[u8]) -> Vec<u8> {
    if let [byte] = payload
        && *byte < 0x80
    {
        return vec![*byte];
    }
    let mut output = Vec::new();
    append_header(0x80, payload.len(), &mut output);
    output.extend_from_slice(payload);
    output
}

fn list(items: &[Vec<u8>]) -> Vec<u8> {
    let payload_len = items.iter().map(Vec::len).sum();
    let mut output = Vec::new();
    append_header(0xc0, payload_len, &mut output);
    for item in items {
        output.extend_from_slice(item);
    }
    output
}

fn append_header(offset: u8, payload_len: usize, output: &mut Vec<u8>) {
    if payload_len < 56 {
        output.push(add_u8(offset, usize_to_u8(payload_len)));
        return;
    }
    let len_bytes = payload_len.to_be_bytes();
    let Some(first) = len_bytes.iter().position(|byte| *byte != 0) else {
        return;
    };
    let len_of_len = usize_to_u8(len_bytes.len().saturating_sub(first));
    output.push(add_u8(add_u8(offset, 55), len_of_len));
    let Some(encoded_len) = len_bytes.get(first..) else {
        return;
    };
    output.extend_from_slice(encoded_len);
}

fn decode_ok(
    input: &[u8],
    field_set: HeaderFieldSet,
) -> Result<UnvalidatedBlockHeader<'_>, &'static str> {
    match decode_block_header(input, field_set, TEST_LIMITS) {
        Ok(header) => Ok(header),
        Err(_) => Err("header fixture should decode"),
    }
}

fn decode_err(
    input: &[u8],
    field_set: HeaderFieldSet,
) -> Result<BlockHeaderDecodeError, &'static str> {
    match decode_block_header(input, field_set, TEST_LIMITS) {
        Ok(_) => Err("header fixture should fail"),
        Err(error) => Ok(error),
    }
}

fn usize_to_u8(value: usize) -> u8 {
    u8::try_from(value).unwrap_or(u8::MAX)
}

fn add_u8(left: u8, right: u8) -> u8 {
    left.saturating_add(right)
}
