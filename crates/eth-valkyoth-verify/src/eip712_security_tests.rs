extern crate std;

use std::vec::Vec;

use super::*;
use crate::test_crypto::RealKeccak;

#[test]
fn borrowed_schema_rejects_invalid_identifiers_before_output_mutation() {
    let invalid_type = [Eip712StructType {
        name: "Amount,address recipient",
        fields: &[],
    }];
    let invalid_field = [Eip712StructType {
        name: "Payment",
        fields: &[Eip712Field {
            name: "amount,address recipient",
            type_name: "uint256",
        }],
    }];
    let mut output = [0xa5_u8; 96];

    assert_eq!(
        encode_eip712_type(&invalid_type, invalid_type[0].name, &mut output),
        Err(Eip712EncodeError::InvalidType)
    );
    assert!(output.iter().all(|byte| *byte == 0xa5));
    assert_eq!(
        encode_eip712_type(&invalid_field, "Payment", &mut output),
        Err(Eip712EncodeError::InvalidType)
    );
    assert!(output.iter().all(|byte| *byte == 0xa5));
}

#[test]
fn borrowed_schema_rejects_duplicate_types_fields_and_values() {
    let duplicate_types = [
        Eip712StructType {
            name: "Payment",
            fields: &[],
        },
        Eip712StructType {
            name: "Payment",
            fields: &[],
        },
    ];
    let duplicate_fields = [Eip712StructType {
        name: "Payment",
        fields: &[
            Eip712Field {
                name: "amount",
                type_name: "uint256",
            },
            Eip712Field {
                name: "amount",
                type_name: "uint256",
            },
        ],
    }];
    let values = [
        Eip712Value {
            name: "amount",
            value: Eip712ValueKind::Uint64(1),
        },
        Eip712Value {
            name: "amount",
            value: Eip712ValueKind::Uint64(2),
        },
    ];
    let mut output = [0u8; 64];
    let mut scratch = [0u8; 96];

    assert_eq!(
        encode_eip712_type(&duplicate_types, "Payment", &mut output),
        Err(Eip712EncodeError::DuplicateType)
    );
    assert_eq!(
        encode_eip712_type(&duplicate_fields, "Payment", &mut output),
        Err(Eip712EncodeError::DuplicateField)
    );
    assert_eq!(
        encode_eip712_data::<RealKeccak>(
            &[Eip712StructType {
                name: "Payment",
                fields: &[Eip712Field {
                    name: "amount",
                    type_name: "uint256",
                }],
            }],
            "Payment",
            &values,
            &mut output,
            &mut scratch,
        ),
        Err(Eip712EncodeError::DuplicateValue)
    );
}

#[test]
fn borrowed_schema_reserves_atomic_looking_struct_names() {
    let mut output = [0u8; 96];
    for name in [
        "uint",
        "int",
        "uint7",
        "uint264",
        "bytes0",
        "bytes33",
        "fixed",
        "ufixed",
        "fixed128x18",
        "ufixed256x80",
    ] {
        let types = [Eip712StructType { name, fields: &[] }];
        assert_eq!(
            encode_eip712_type(&types, name, &mut output),
            Err(Eip712EncodeError::InvalidType),
            "{name}"
        );
    }

    let types = [Eip712StructType {
        name: "uintToken",
        fields: &[],
    }];
    assert!(encode_eip712_type(&types, "uintToken", &mut output).is_ok());
}

#[test]
fn borrowed_schema_bounds_fields_and_values_before_duplicate_scans() {
    let fields = [Eip712Field {
        name: "value",
        type_name: "uint256",
    }; EIP712_MAX_FIELDS_PER_TYPE + 1];
    let types = [Eip712StructType {
        name: "Oversized",
        fields: &fields,
    }];
    let mut output = [0u8; 96];
    assert_eq!(
        encode_eip712_type(&types, "Oversized", &mut output),
        Err(Eip712EncodeError::ResourceLimit)
    );

    let valid_types = [Eip712StructType {
        name: "Value",
        fields: &[],
    }];
    let values = (0..=EIP712_MAX_VALUES_PER_STRUCT)
        .map(|_| Eip712Value {
            name: "value",
            value: Eip712ValueKind::Uint64(1),
        })
        .collect::<Vec<_>>();
    let mut scratch = [0u8; 96];
    assert_eq!(
        encode_eip712_data::<RealKeccak>(&valid_types, "Value", &values, &mut output, &mut scratch,),
        Err(Eip712EncodeError::ResourceLimit)
    );
}

#[test]
fn failed_data_encoding_clears_partially_written_output() {
    let types = [Eip712StructType {
        name: "Payment",
        fields: &[
            Eip712Field {
                name: "amount",
                type_name: "uint256",
            },
            Eip712Field {
                name: "recipient",
                type_name: "address",
            },
        ],
    }];
    let values = [Eip712Value {
        name: "amount",
        value: Eip712ValueKind::Uint64(7),
    }];
    let mut output = [0xa5_u8; 64];
    let mut scratch = [0u8; 96];

    assert_eq!(
        encode_eip712_data::<RealKeccak>(&types, "Payment", &values, &mut output, &mut scratch,),
        Err(Eip712EncodeError::MissingValue)
    );
    assert_eq!(output, [0u8; 64]);
}
