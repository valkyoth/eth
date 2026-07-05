extern crate std;
use std::boxed::Box;
use std::string::String;
use std::vec::Vec;

use super::*;
use crate::{
    EthereumSignature, VerifyError, recover_sender_from_digest_with_backend,
    test_crypto::{RealKeccak, TestSecp256k1Backend},
};

const EIP712_SIGNATURE: &str = "0x4355c47d63924e8a72e509b65029052eb6c299d53a04e167c5775fd466751c9d07299936d304c153f6443dfa05f40ff007d72911b6f72307f996231605b915621c";
const EIP712_SIGNER: &str = "0xCD2a3d9F938E13CD947Ec05AbC7FE734Df8DD826";

#[test]
fn encodes_official_eip712_mail_type() -> Result<(), Eip712EncodeError> {
    let mut scratch = [0_u8; 160];
    let len = encode_eip712_type(mail_types(), "Mail", &mut scratch)?;
    let encoded = core::str::from_utf8(
        scratch
            .get(..len)
            .ok_or(Eip712EncodeError::OutputTooShort)?,
    )
    .map_err(|_| Eip712EncodeError::InvalidType)?;

    assert_eq!(
        encoded,
        "Mail(Person from,Person to,string contents)Person(string name,address wallet)"
    );
    Ok(())
}

#[test]
fn recovers_official_eip712_mail_signer() -> Result<(), VerifyError> {
    let mut scratch = [0_u8; 192];
    let digest = eip712_typed_data_signing_digest::<RealKeccak>(
        mail_domain(),
        mail_types(),
        "Mail",
        &mail_message()?,
        &mut scratch,
    )
    .map_err(|_| VerifyError::InvalidSignature)?;
    let signature = decode_eip712_signature(EIP712_SIGNATURE)?;
    let expected = decode_address(EIP712_SIGNER)?;

    assert_eq!(
        recover_sender_from_digest_with_backend(
            digest,
            signature,
            TestSecp256k1Backend,
            RealKeccak::default()
        ),
        Ok(expected)
    );
    Ok(())
}

#[test]
fn encodes_arrays_and_fixed_bytes_as_bounded_words() -> Result<(), Eip712EncodeError> {
    let types = [Eip712StructType {
        name: "Batch",
        fields: &[
            Eip712Field {
                name: "amounts",
                type_name: "uint256[]",
            },
            Eip712Field {
                name: "tag",
                type_name: "bytes2",
            },
            Eip712Field {
                name: "active",
                type_name: "bool",
            },
        ],
    }];
    let amounts = [Eip712ValueKind::Uint64(1), Eip712ValueKind::Uint64(2)];
    let values = [
        Eip712Value {
            name: "amounts",
            value: Eip712ValueKind::Array(&amounts),
        },
        Eip712Value {
            name: "tag",
            value: Eip712ValueKind::FixedBytes(&[0xab, 0xcd]),
        },
        Eip712Value {
            name: "active",
            value: Eip712ValueKind::Bool(true),
        },
    ];
    let mut type_scratch = [0_u8; 96];
    let mut encoded = [0_u8; 96];

    assert_eq!(
        encode_eip712_data::<RealKeccak>(&types, "Batch", &values, &mut encoded, &mut type_scratch),
        Ok(96)
    );
    assert_eq!(encoded.get(32), Some(&0xab));
    assert_eq!(encoded.get(33), Some(&0xcd));
    assert_eq!(encoded.last(), Some(&1));
    Ok(())
}

#[test]
fn rejects_missing_values_and_type_mismatches() {
    let mut type_scratch = [0_u8; 128];
    let mut output = [0_u8; 96];
    assert_eq!(
        encode_eip712_data::<RealKeccak>(mail_types(), "Mail", &[], &mut output, &mut type_scratch),
        Err(Eip712EncodeError::MissingValue)
    );

    let types = [Eip712StructType {
        name: "Bad",
        fields: &[Eip712Field {
            name: "tag",
            type_name: "bytes4",
        }],
    }];
    let values = [Eip712Value {
        name: "tag",
        value: Eip712ValueKind::FixedBytes(&[1, 2]),
    }];
    assert_eq!(
        encode_eip712_data::<RealKeccak>(&types, "Bad", &values, &mut output, &mut type_scratch),
        Err(Eip712EncodeError::TypeMismatch)
    );
}

#[test]
fn rejects_reserved_atomic_struct_name_collision() {
    let types = [
        Eip712StructType {
            name: "Person",
            fields: &[Eip712Field {
                name: "wallet",
                type_name: "address",
            }],
        },
        Eip712StructType {
            name: "address",
            fields: &[Eip712Field {
                name: "evil",
                type_name: "uint256",
            }],
        },
    ];
    let mut scratch = [0_u8; 128];

    assert_eq!(
        encode_eip712_type(&types, "Person", &mut scratch),
        Err(Eip712EncodeError::InvalidType)
    );
    assert_eq!(
        encode_eip712_type(&types, "address", &mut scratch),
        Err(Eip712EncodeError::InvalidType)
    );
}

#[test]
fn rejects_array_dimensionality_over_recursion_limit() {
    let mut type_name = String::from("uint256");
    for _ in 0..MAX_TYPE_DEPTH {
        type_name.push_str("[]");
    }
    let type_name = Box::leak(type_name.into_boxed_str());
    let types = [Eip712StructType {
        name: "Deep",
        fields: &[Eip712Field {
            name: "value",
            type_name,
        }],
    }];
    let values = [Eip712Value {
        name: "value",
        value: nested_array_value(MAX_TYPE_DEPTH),
    }];
    let mut scratch = [0_u8; 128];

    assert_eq!(
        eip712_hash_struct::<RealKeccak>(&types, "Deep", &values, &mut scratch),
        Err(Eip712EncodeError::RecursionLimit)
    );
}

fn mail_types<'a>() -> &'a [Eip712StructType<'a>] {
    &[
        Eip712StructType {
            name: "Mail",
            fields: &[
                Eip712Field {
                    name: "from",
                    type_name: "Person",
                },
                Eip712Field {
                    name: "to",
                    type_name: "Person",
                },
                Eip712Field {
                    name: "contents",
                    type_name: "string",
                },
            ],
        },
        Eip712StructType {
            name: "Person",
            fields: &[
                Eip712Field {
                    name: "name",
                    type_name: "string",
                },
                Eip712Field {
                    name: "wallet",
                    type_name: "address",
                },
            ],
        },
    ]
}

fn mail_domain() -> Eip712DomainData<'static> {
    Eip712DomainData {
        name: Some("Ether Mail"),
        version: Some("1"),
        chain_id: Some(ChainId::new(1)),
        verifying_contract: Some(Address::from_bytes([0xcc_u8; 20])),
        salt: None,
    }
}

fn mail_message<'a>() -> Result<[Eip712Value<'a>; 3], VerifyError> {
    let cow = [
        Eip712Value {
            name: "name",
            value: Eip712ValueKind::String("Cow"),
        },
        Eip712Value {
            name: "wallet",
            value: Eip712ValueKind::Address(decode_address(EIP712_SIGNER)?),
        },
    ];
    let bob = [
        Eip712Value {
            name: "name",
            value: Eip712ValueKind::String("Bob"),
        },
        Eip712Value {
            name: "wallet",
            value: Eip712ValueKind::Address(Address::from_bytes([0xbb_u8; 20])),
        },
    ];
    Ok([
        Eip712Value {
            name: "from",
            value: Eip712ValueKind::Struct(Box::leak(Box::new(cow))),
        },
        Eip712Value {
            name: "to",
            value: Eip712ValueKind::Struct(Box::leak(Box::new(bob))),
        },
        Eip712Value {
            name: "contents",
            value: Eip712ValueKind::String("Hello, Bob!"),
        },
    ])
}

fn nested_array_value(depth: usize) -> Eip712ValueKind<'static> {
    if depth == 0 {
        return Eip712ValueKind::Uint64(1);
    }
    let child = [nested_array_value(depth.saturating_sub(1))];
    Eip712ValueKind::Array(Box::leak(Box::new(child)))
}

fn decode_eip712_signature(input: &str) -> Result<EthereumSignature, VerifyError> {
    let mut bytes = decode_hex(input).map_err(|_| VerifyError::InvalidSignature)?;
    let recovery = bytes.last_mut().ok_or(VerifyError::InvalidSignature)?;
    if *recovery >= 27 {
        *recovery = recovery.saturating_sub(27);
    }
    let bytes =
        <[u8; 65]>::try_from(bytes.as_slice()).map_err(|_| VerifyError::InvalidSignature)?;
    EthereumSignature::try_from_bytes(bytes)
}

fn decode_address(input: &str) -> Result<Address, VerifyError> {
    let bytes = decode_hex(input).map_err(|_| VerifyError::InvalidSignature)?;
    let bytes =
        <[u8; 20]>::try_from(bytes.as_slice()).map_err(|_| VerifyError::InvalidSignature)?;
    Ok(Address::from_bytes(bytes))
}

fn decode_hex(input: &str) -> Result<Vec<u8>, ()> {
    let hex = input.strip_prefix("0x").unwrap_or(input);
    if !hex.len().is_multiple_of(2) {
        return Err(());
    }
    let bytes = hex.as_bytes();
    let mut output = Vec::with_capacity(bytes.len() / 2);
    for chunk in bytes.chunks_exact(2) {
        let high = hex_nibble(chunk.first().copied().ok_or(())?)?;
        let low = hex_nibble(chunk.get(1).copied().ok_or(())?)?;
        output.push((high << 4) | low);
    }
    Ok(output)
}

fn hex_nibble(byte: u8) -> Result<u8, ()> {
    match byte {
        b'0'..=b'9' => byte.checked_sub(b'0').ok_or(()),
        b'a'..=b'f' => byte
            .checked_sub(b'a')
            .and_then(|value| value.checked_add(10))
            .ok_or(()),
        b'A'..=b'F' => byte
            .checked_sub(b'A')
            .and_then(|value| value.checked_add(10))
            .ok_or(()),
        _ => Err(()),
    }
}
