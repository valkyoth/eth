use eth_valkyoth_codec::DecodeLimits;
use eth_valkyoth_hash::Keccak256Digest;
use eth_valkyoth_primitives::{Address, B256, ChainId};
use eth_valkyoth_protocol::{
    decode_access_list_transaction, decode_blob_transaction, decode_dynamic_fee_transaction,
};
use sha3::Digest;
extern crate std;
use std::vec::Vec;

use super::*;

const KAT_LIMITS: DecodeLimits = DecodeLimits {
    max_input_bytes: 4096,
    max_list_items: 256,
    max_nesting_depth: 16,
    max_total_allocation: 4096,
    max_proof_nodes: 4,
    max_total_items: 512,
};

// Source: ethereum.publicnode.com, eth_getRawTransactionByHash for
// 0xeb8535da9d096a3be85faefd347feddaa82668ca4ef199203450065ef1c28d39.
// The expected sender is the RPC `from` field for the same transaction.
const ACCESS_LIST_RAW: &str = "0x01f9028f018220038512a05f200083124f809490d57c386e6403bf3473a1001d248d43eb85f84d80b90224a2abe54e00000000000000000000000000000000000000000000000050cbcc988bc3b3f700000000000000000000000000000000000000000000000000581b77f66e000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000600000000000000000000000000000000000000000000000000000000000000c00000000000000000000000000000000000000000000000000000000000000002000000000000000000000000570febdf89c07f256c75686caca215289bb11cfc000000000000000000000000bbc2ae13b23d715c30720f079fcd9b4a74093505000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000026f2000000000000000000000000000000000000000000000000000000000000271000000000000000000000000007ed78c6c91ce18811ad281d0533819cf848075b000000000000000000000000c02aaa39b223fe8d0a0e5c4f27ead9083c756cc2000000000000000000000000000000000000000000000000000000000000000a00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000002710c080a028cf96d0c6cbbe7ec05e1f50e83805c3dd93d47afcb4c566b4d93f9baa628228a0081abbd4a24c5ef0f21b2cbddf2d934506eecf9b1a4cc8808dff6b8ebb1ecec5";
const ACCESS_LIST_SENDER: &str = "0xd006ad343886254359dc87aa5e1753446072fd50";

// Source: ethereum.publicnode.com, eth_getRawTransactionByHash for
// 0x6ee06d2bf01790bd98d068767321bd997df45352f126267143c13256e1623c5c.
// The expected sender is the RPC `from` field for the same transaction.
const DYNAMIC_FEE_RAW: &str = "0x02f90158018301097a8085093bcfd39a8307a120947a250d5630b4cf539739df2c5dacb4c659f2488d8809cfc4c110338000b8e47ff36ab500000000000000000000000000000000000000000000000000002c73c7b52a4000000000000000000000000000000000000000000000000000000000000000800000000000000000000000005daaf5eded5cc8daba8c5a28f7bb87efc2a7916e00000000000000000000000000000000000000000000000000000000673ba5570000000000000000000000000000000000000000000000000000000000000002000000000000000000000000c02aaa39b223fe8d0a0e5c4f27ead9083c756cc2000000000000000000000000446c9033e7516d820cc9a2ce2d0b7328b579406fc001a0d2a93b30d8319e709f5a59e2f270074e4afc773cd574ca0d42f8394dc55b1858a06e1ae8fe2c946620b809bd2950eac1b9a56b5864b11b8a6666b76377b69463ad";
const DYNAMIC_FEE_SENDER: &str = "0x9696a5c3eb572de180aa7f76e39c0f4418a34af1";

// Source: ethereum.publicnode.com, eth_getRawTransactionByHash for
// 0x0ff07f37baa7fa26bb7de3d3fc63002bf0acf3295bdab7f67c108c0d1a3bff15.
// The expected sender is the RPC `from` field for the same transaction.
const BLOB_RAW: &str = "0x03f905e0018250d685012a05f200850826299e00832dc6c09406a9ab27c7e2255df1815e6cc0168d7755feb19a843b9aca00b90544ef16e8450000000000000000000000000000000000000000000000000000000000000040000000000000000000000000000000000000000000000000000000000000052000000000000000000000000000000000000000000000000000000000000004c0000000000000000000000000000000000000000000000000000000000000002000000000000000000000000068d30f47f19c07bccef4ac7fae2dc12fca3e0dc9000000000000000000000000000000633b68f5d8d3a86593ebb815b4663bcbe0302e31382e302d64657600000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000c0000000000000000000000000000000000000000000000000000000000000042000000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000020000000000000000000000000537a2f0d3a5879b41bcb5a2afe2ea5c4961796f6000000000000000000000000000000000000000000000000000000000000004000000000000000000000000000000000000000000000000000000000000002c0000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000000400000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000665ba98500000000000000000000000000000000000000000000000000000000013130ff000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000001e0000000000000000000000000000000000000000000000000000000000000000300000000000000000000000000000000000000000000000000000000000000c8000000000000000000000000000000000000000000000000000000003b9aca000000000000000000000000000000000000000000000000000000000000000384000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000003e80000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000004104204bcc82b3c4237f7c87389a741a1a89d6607897e67b9e2838c52ff30ca6a11c589d058342e9f538e27d2fae89cad91e00f80e531390f3f4ffa49419fd15221b000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000041f6f0daa2bf2af011c65f6b01aa6902ca099ff5fbdad194397ad4bd32b7b8d6482a9996b1682020433a79662551da6ae69510daef27b83f00d7f59796667cd6261c000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000c0843b9aca00e1a0017ba4bd9c166498865a3d08618e333ee84812941b5c3a356971b4a6ffffa57401a079d49cd5724eb7194af4202b59a25e9782d3bd6cb8f20e7049dd0204c8ff58e8a0662fb12590d7121243aaddf9d39ab8231758abbfe84df53805750fd40db6c1ce";
const BLOB_SENDER: &str = "0x000000633b68f5d8d3a86593ebb815b4663bcbe0";

struct RealKeccak {
    inner: sha3::Keccak256,
}

impl RealKeccak {
    fn new() -> Self {
        Self {
            inner: sha3::Keccak256::new(),
        }
    }
}

impl Keccak256 for RealKeccak {
    fn update(&mut self, input: &[u8]) {
        Digest::update(&mut self.inner, input);
    }

    fn finalize(self) -> Keccak256Digest {
        B256::from_bytes(self.inner.finalize().into())
    }
}

#[test]
fn validates_external_typed_transaction_kats() {
    validates_access_list_kat();
    validates_dynamic_fee_kat();
    validates_blob_kat();
}

fn validates_access_list_kat() {
    let raw = decode_hex(ACCESS_LIST_RAW);
    let expected = decode_address(ACCESS_LIST_SENDER);
    assert!(raw.is_ok());
    assert!(expected.is_ok());
    if let (Ok(raw), Ok(expected)) = (raw, expected) {
        let tx = decode_access_list_transaction(&raw, KAT_LIMITS);
        assert!(tx.is_ok(), "{tx:?}");
        if let Ok(tx) = tx {
            let mut scratch = [0_u8; 4096];
            assert_eq!(
                validate_access_list_transaction_signature(
                    ChainId::new(1),
                    &tx,
                    Some(expected),
                    &mut scratch,
                    RealKeccak::new(),
                    RealKeccak::new(),
                )
                .map(ValidatedTransactionSignature::sender),
                Ok(expected)
            );
        }
    }
}

fn validates_dynamic_fee_kat() {
    let raw = decode_hex(DYNAMIC_FEE_RAW);
    let expected = decode_address(DYNAMIC_FEE_SENDER);
    assert!(raw.is_ok());
    assert!(expected.is_ok());
    if let (Ok(raw), Ok(expected)) = (raw, expected) {
        let tx = decode_dynamic_fee_transaction(&raw, KAT_LIMITS);
        assert!(tx.is_ok(), "{tx:?}");
        if let Ok(tx) = tx {
            let mut scratch = [0_u8; 4096];
            assert_eq!(
                validate_dynamic_fee_transaction_signature(
                    ChainId::new(1),
                    &tx,
                    Some(expected),
                    &mut scratch,
                    RealKeccak::new(),
                    RealKeccak::new(),
                )
                .map(ValidatedTransactionSignature::sender),
                Ok(expected)
            );
        }
    }
}

fn validates_blob_kat() {
    let raw = decode_hex(BLOB_RAW);
    let expected = decode_address(BLOB_SENDER);
    assert!(raw.is_ok());
    assert!(expected.is_ok());
    if let (Ok(raw), Ok(expected)) = (raw, expected) {
        let tx = decode_blob_transaction(&raw, KAT_LIMITS);
        assert!(tx.is_ok(), "{tx:?}");
        if let Ok(tx) = tx {
            let mut scratch = [0_u8; 4096];
            assert_eq!(
                validate_blob_transaction_signature(
                    ChainId::new(1),
                    &tx,
                    Some(expected),
                    &mut scratch,
                    RealKeccak::new(),
                    RealKeccak::new(),
                )
                .map(ValidatedTransactionSignature::sender),
                Ok(expected)
            );
        }
    }
}

fn decode_address(input: &str) -> Result<Address, ()> {
    let bytes = decode_hex(input)?;
    let bytes = <[u8; 20]>::try_from(bytes.as_slice()).map_err(|_| ())?;
    Ok(Address::from_bytes(bytes))
}

fn decode_hex(input: &str) -> Result<Vec<u8>, ()> {
    let hex = match input.strip_prefix("0x") {
        Some(value) => value,
        None => input,
    };
    let bytes = hex.as_bytes();
    let mut chunks = bytes.chunks_exact(2);
    if !chunks.remainder().is_empty() {
        return Err(());
    }
    let mut output = Vec::with_capacity(bytes.len() / 2);
    for chunk in &mut chunks {
        let high = chunk.first().copied().ok_or(())?;
        let low = chunk.get(1).copied().ok_or(())?;
        output.push((hex_nibble(high)? << 4) | hex_nibble(low)?);
    }
    Ok(output)
}

fn hex_nibble(value: u8) -> Result<u8, ()> {
    match value {
        b'0'..=b'9' => value.checked_sub(b'0').ok_or(()),
        b'a'..=b'f' => value
            .checked_sub(b'a')
            .and_then(|nibble| nibble.checked_add(10))
            .ok_or(()),
        b'A'..=b'F' => value
            .checked_sub(b'A')
            .and_then(|nibble| nibble.checked_add(10))
            .ok_or(()),
        _ => Err(()),
    }
}
