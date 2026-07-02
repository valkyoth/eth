use eth_valkyoth_hash::Keccak256Digest;
use sha3::Digest;

use super::*;

struct RealKeccak {
    inner: sha3::Keccak256,
}

impl Default for RealKeccak {
    fn default() -> Self {
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
fn parses_ether_mail_json() {
    let mut scratch = [0_u8; 512];
    let digest = eip712_json_typed_data_signing_digest::<RealKeccak>(
        ether_mail_json(),
        Eip712JsonLimits::DEFAULT,
        &mut scratch,
    );

    assert!(digest.is_ok(), "{digest:?}");
}

#[test]
fn rejects_duplicate_type_key() {
    let mut scratch = [0_u8; 512];
    let json = r#"{
        "types": {"Person": [], "Person": []},
        "primaryType": "Person",
        "domain": {},
        "message": {}
    }"#;

    assert_eq!(
        eip712_json_typed_data_signing_digest::<RealKeccak>(
            json,
            Eip712JsonLimits::DEFAULT,
            &mut scratch,
        ),
        Err(Eip712JsonError::Json)
    );
}

#[test]
fn rejects_missing_primary_type() {
    let mut scratch = [0_u8; 512];
    let json = r#"{"types": {}, "domain": {}, "message": {}}"#;

    assert_eq!(
        eip712_json_typed_data_signing_digest::<RealKeccak>(
            json,
            Eip712JsonLimits::DEFAULT,
            &mut scratch,
        ),
        Err(Eip712JsonError::Shape)
    );
}

fn ether_mail_json() -> &'static str {
    r#"{
        "types": {
            "EIP712Domain": [
                {"name": "name", "type": "string"},
                {"name": "version", "type": "string"},
                {"name": "chainId", "type": "uint256"},
                {"name": "verifyingContract", "type": "address"}
            ],
            "Person": [
                {"name": "name", "type": "string"},
                {"name": "wallet", "type": "address"}
            ],
            "Mail": [
                {"name": "from", "type": "Person"},
                {"name": "to", "type": "Person"},
                {"name": "contents", "type": "string"}
            ]
        },
        "primaryType": "Mail",
        "domain": {
            "name": "Ether Mail",
            "version": "1",
            "chainId": 1,
            "verifyingContract": "0xCcCCccccCCCCcCCCCCCcCcCccCcCCCcCcccccccC"
        },
        "message": {
            "from": {
                "name": "Cow",
                "wallet": "0xCD2a3d9F938E13CD947Ec05AbC7FE734Df8DD826"
            },
            "to": {
                "name": "Bob",
                "wallet": "0xbBbBBBBbbBBBbbbBbbBbbbbBBbBbbbbBbBbbBBbB"
            },
            "contents": "Hello, Bob!"
        }
    }"#
}
