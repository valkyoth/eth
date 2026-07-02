use eth_valkyoth_hash::Keccak256Digest;
use sha3::Digest;
use std::format;

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

#[test]
fn parses_full_width_signed_decimal_strings() {
    let mut scratch = [0_u8; 512];
    for json in [
        json_for_field("int64", r#""-9223372036854775808""#),
        json_for_field("int256", r#""-100000000000000000000""#),
        json_for_field("int256", r#""18446744073709551616""#),
    ] {
        let digest = eip712_json_typed_data_signing_digest::<RealKeccak>(
            &json,
            Eip712JsonLimits::DEFAULT,
            &mut scratch,
        );
        assert!(digest.is_ok(), "{digest:?}");
    }
}

#[test]
fn rejects_decimal_integer_boundaries() {
    assert_json_error(
        &json_for_field("uint256", r#""""#),
        Eip712JsonLimits::DEFAULT,
        Eip712JsonError::Integer,
    );
    assert_json_error(
        &json_for_field("int256", r#""-""#),
        Eip712JsonLimits::DEFAULT,
        Eip712JsonError::Integer,
    );
    assert_json_error(
        &json_for_field(
            "int256",
            r#""57896044618658097711785492504343953926634992332820282019728792003956564819968""#,
        ),
        Eip712JsonLimits::DEFAULT,
        Eip712JsonError::Integer,
    );
    assert_json_error(
        &json_for_field(
            "int256",
            r#""-57896044618658097711785492504343953926634992332820282019728792003956564819969""#,
        ),
        Eip712JsonLimits::DEFAULT,
        Eip712JsonError::Integer,
    );
}

#[test]
fn validates_domain_shape_and_domain_fields() {
    assert_json_error(
        r#"{
            "types": {"EIP712Domain": "not an array", "Value": []},
            "primaryType": "Value",
            "domain": {},
            "message": {}
        }"#,
        Eip712JsonLimits::DEFAULT,
        Eip712JsonError::Shape,
    );
    assert_json_error(
        r#"{
            "types": {"Value": []},
            "primaryType": "Value",
            "domain": {"chainId": 0},
            "message": {}
        }"#,
        Eip712JsonLimits::DEFAULT,
        Eip712JsonError::Integer,
    );
    assert_json_error(
        r#"{
            "types": {"Value": []},
            "primaryType": "Value",
            "domain": {"unknown": "field"},
            "message": {}
        }"#,
        Eip712JsonLimits::DEFAULT,
        Eip712JsonError::Shape,
    );
}

#[test]
fn enforces_configured_limits() {
    assert_json_error(
        &json_for_field("uint256", "1"),
        Eip712JsonLimits {
            max_input_bytes: 2,
            ..Eip712JsonLimits::DEFAULT
        },
        Eip712JsonError::Limit,
    );
    assert_json_error(
        r#"{
            "types": {"A": [], "B": []},
            "primaryType": "A",
            "domain": {},
            "message": {}
        }"#,
        Eip712JsonLimits {
            max_types: 1,
            ..Eip712JsonLimits::DEFAULT
        },
        Eip712JsonError::Limit,
    );
    assert_json_error(
        r#"{
            "types": {
                "Value": [
                    {"name": "one", "type": "uint256"},
                    {"name": "two", "type": "uint256"}
                ]
            },
            "primaryType": "Value",
            "domain": {},
            "message": {"one": 1, "two": 2}
        }"#,
        Eip712JsonLimits {
            max_fields_per_type: 1,
            ..Eip712JsonLimits::DEFAULT
        },
        Eip712JsonError::Limit,
    );
    assert_json_error(
        &json_for_field("uint256[]", "[1, 2]"),
        Eip712JsonLimits {
            max_array_items: 1,
            ..Eip712JsonLimits::DEFAULT
        },
        Eip712JsonError::Limit,
    );
    assert_json_error(
        &json_for_field("string", r#""abcd""#),
        Eip712JsonLimits {
            max_string_bytes: 3,
            ..Eip712JsonLimits::DEFAULT
        },
        Eip712JsonError::Limit,
    );
    assert_json_error(
        &json_for_field("bytes", r#""0x0000""#),
        Eip712JsonLimits {
            max_bytes_value: 1,
            ..Eip712JsonLimits::DEFAULT
        },
        Eip712JsonError::Limit,
    );
    assert_json_error(
        r#"{
            "types": {
                "A": [{"name": "b", "type": "B"}],
                "B": [{"name": "value", "type": "uint256"}]
            },
            "primaryType": "A",
            "domain": {},
            "message": {"b": {"value": 1}}
        }"#,
        Eip712JsonLimits {
            max_depth: 1,
            ..Eip712JsonLimits::DEFAULT
        },
        Eip712JsonError::Encode(Eip712EncodeError::RecursionLimit),
    );
}

#[test]
fn rejects_shape_and_hex_edge_cases() {
    assert_json_error(
        &json_for_field("uint256[2]", "[1]"),
        Eip712JsonLimits::DEFAULT,
        Eip712JsonError::Shape,
    );
    assert_json_error(
        r#"{
            "types": {
                "Value": [
                    {"name": "same", "type": "uint256"},
                    {"name": "same", "type": "uint256"}
                ]
            },
            "primaryType": "Value",
            "domain": {},
            "message": {"same": 1}
        }"#,
        Eip712JsonLimits::DEFAULT,
        Eip712JsonError::Shape,
    );
    assert_json_error(
        &json_for_field("bytes2", r#""0x0""#),
        Eip712JsonLimits::DEFAULT,
        Eip712JsonError::Hex,
    );
    assert_json_error(
        &json_for_field("bytes2", r#""0xzz""#),
        Eip712JsonLimits::DEFAULT,
        Eip712JsonError::Hex,
    );
}

#[test]
fn fixed_bytes_are_not_restricted_by_dynamic_bytes_limit() {
    let mut scratch = [0_u8; 512];
    let value = format!(r#""0x{}""#, "11".repeat(32));
    let json = json_for_field("bytes32", &value);
    let digest = eip712_json_typed_data_signing_digest::<RealKeccak>(
        &json,
        Eip712JsonLimits {
            max_bytes_value: 1,
            ..Eip712JsonLimits::DEFAULT
        },
        &mut scratch,
    );

    assert!(digest.is_ok(), "{digest:?}");
}

#[test]
fn rejects_excessively_wide_json_objects() {
    let mut json = String::from("{");
    for index in 0..513 {
        if index != 0 {
            json.push(',');
        }
        json.push('"');
        json.push_str("key");
        json.push_str(&index.to_string());
        json.push_str(r#"":0"#);
    }
    json.push('}');

    assert_json_error(&json, Eip712JsonLimits::DEFAULT, Eip712JsonError::Json);
}

#[test]
fn rejects_raw_json_structural_depth_before_parser_walk() {
    let mut json =
        String::from(r#"{"types":{"Value":[]},"primaryType":"Value","domain":{},"message":"#);
    for _ in 0..2048 {
        json.push('[');
    }
    json.push('0');
    for _ in 0..2048 {
        json.push(']');
    }
    json.push('}');

    assert_json_error(
        &json,
        Eip712JsonLimits {
            max_input_bytes: json.len().saturating_add(1),
            ..Eip712JsonLimits::DEFAULT
        },
        Eip712JsonError::Json,
    );
}

fn assert_json_error(json: &str, limits: Eip712JsonLimits, expected: Eip712JsonError) {
    let mut scratch = [0_u8; 512];
    assert_eq!(
        eip712_json_typed_data_signing_digest::<RealKeccak>(json, limits, &mut scratch),
        Err(expected)
    );
}

fn json_for_field(field_type: &str, value: &str) -> String {
    format!(
        r#"{{
            "types": {{"Value": [{{"name": "value", "type": "{field_type}"}}]}},
            "primaryType": "Value",
            "domain": {{}},
            "message": {{"value": {value}}}
        }}"#
    )
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
