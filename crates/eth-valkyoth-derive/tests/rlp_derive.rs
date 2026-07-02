#![allow(missing_docs)]

use eth_valkyoth_codec::{DecodeLimits, RlpDecode as _, RlpEncode as _};
use eth_valkyoth_derive::{RlpDecode, RlpEncode};
use eth_valkyoth_primitives::{Address, ChainId, Nonce};

const TEST_LIMITS: DecodeLimits = DecodeLimits {
    max_input_bytes: 1024,
    max_list_items: 16,
    max_nesting_depth: 4,
    max_total_allocation: 1024,
    max_proof_nodes: 1,
    max_total_items: 64,
};

#[derive(Debug, Eq, PartialEq, RlpDecode, RlpEncode)]
struct Transfer {
    chain_id: ChainId,
    nonce: Nonce,
    to: Address,
    #[eth_rlp(skip, default, reason = "derived cache")]
    cached_sender: [u8; 20],
}

#[derive(Debug, Eq, PartialEq, RlpDecode, RlpEncode)]
struct Pair(u64, [u8; 2]);

#[derive(Debug, Eq, PartialEq, RlpDecode, RlpEncode)]
struct Empty;

#[test]
fn rlp_derives_round_trip_named_struct_with_skipped_default()
-> Result<(), eth_valkyoth_codec::RlpDeriveError> {
    let transfer = Transfer {
        chain_id: ChainId::new(1),
        nonce: Nonce::new(2),
        to: Address::from_bytes([0x11; 20]),
        cached_sender: [0x22; 20],
    };
    let mut output = [0u8; 64];
    let written = transfer.encode_rlp(&mut output)?;
    let encoded = output
        .get(..written)
        .ok_or(eth_valkyoth_codec::RlpDeriveError::Decode(
            eth_valkyoth_codec::DecodeError::OffsetOutOfBounds,
        ))?;

    let expected = [
        0xd7, 0x01, 0x02, 0x94, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11,
        0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11,
    ];

    assert_eq!(encoded, expected);
    let decoded = Transfer::decode_rlp(encoded, TEST_LIMITS)?;
    assert_eq!(
        decoded,
        Transfer {
            cached_sender: [0u8; 20],
            ..transfer
        }
    );
    Ok(())
}

#[test]
fn rlp_derives_round_trip_tuple_and_unit_structs() -> Result<(), eth_valkyoth_codec::RlpDeriveError>
{
    let pair = Pair(15, [0xaa, 0xbb]);
    let mut pair_output = [0u8; 8];
    let pair_written = pair.encode_rlp(&mut pair_output)?;
    let pair_encoded =
        pair_output
            .get(..pair_written)
            .ok_or(eth_valkyoth_codec::RlpDeriveError::Decode(
                eth_valkyoth_codec::DecodeError::OffsetOutOfBounds,
            ))?;
    assert_eq!(Pair::decode_rlp(pair_encoded, TEST_LIMITS)?, pair);

    let empty = Empty;
    let mut empty_output = [0u8; 1];
    assert_eq!(empty.encode_rlp(&mut empty_output)?, 1);
    assert_eq!(empty_output, [0xc0]);
    assert_eq!(Empty::decode_rlp(&empty_output, TEST_LIMITS)?, empty);
    Ok(())
}

#[test]
fn rlp_derive_compile_failures_are_stable() {
    let tests = trybuild::TestCases::new();
    tests.compile_fail("tests/ui/rlp_enum.rs");
    tests.compile_fail("tests/ui/rlp_generic.rs");
    tests.compile_fail("tests/ui/rlp_skip_without_reason.rs");
}
