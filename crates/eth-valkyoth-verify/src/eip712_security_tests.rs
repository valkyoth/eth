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
