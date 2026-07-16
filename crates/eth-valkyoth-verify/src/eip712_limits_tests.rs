extern crate std;

use super::*;
use crate::test_crypto::RealKeccak;
use std::vec;

#[test]
fn default_dynamic_byte_budget_rejects_one_oversized_value() {
    let oversized = vec![0_u8; EIP712_MAX_DYNAMIC_VALUE_BYTES.saturating_add(1)];
    let types = [Eip712StructType {
        name: "Payload",
        fields: &[Eip712Field {
            name: "data",
            type_name: "bytes",
        }],
    }];
    let values = [Eip712Value {
        name: "data",
        value: Eip712ValueKind::Bytes(&oversized),
    }];
    let mut scratch = [0u8; 96];

    assert_eq!(
        eip712_hash_struct::<RealKeccak>(&types, "Payload", &values, &mut scratch),
        Err(Eip712EncodeError::ResourceLimit)
    );
}

#[test]
fn cumulative_dynamic_byte_budget_covers_reused_values_and_domain_strings() {
    let shared = [0x5a_u8; 3];
    let types = [Eip712StructType {
        name: "Payload",
        fields: &[
            Eip712Field {
                name: "first",
                type_name: "bytes",
            },
            Eip712Field {
                name: "second",
                type_name: "bytes",
            },
        ],
    }];
    let values = [
        Eip712Value {
            name: "first",
            value: Eip712ValueKind::Bytes(&shared),
        },
        Eip712Value {
            name: "second",
            value: Eip712ValueKind::Bytes(&shared),
        },
    ];
    let limits = Eip712Limits {
        max_value_nodes: EIP712_MAX_VALUE_NODES,
        max_dynamic_value_bytes: 5,
    };
    let mut scratch = [0u8; 128];

    assert_eq!(
        eip712_hash_struct_with_limits::<RealKeccak>(
            &types,
            "Payload",
            &values,
            limits,
            &mut scratch,
        ),
        Err(Eip712EncodeError::ResourceLimit)
    );

    let domain = Eip712DomainData {
        name: Some("abc"),
        version: Some("def"),
        chain_id: None,
        verifying_contract: None,
        salt: None,
    };
    let empty_types = [Eip712StructType {
        name: "Empty",
        fields: &[],
    }];
    assert_eq!(
        eip712_typed_data_signing_digest_with_limits::<RealKeccak>(
            domain,
            &empty_types,
            "Empty",
            &[],
            limits,
            &mut scratch,
        ),
        Err(Eip712EncodeError::ResourceLimit)
    );
}

#[test]
fn dynamic_byte_budget_accepts_exact_boundary_and_clears_partial_output() {
    let types = [Eip712StructType {
        name: "Payload",
        fields: &[
            Eip712Field {
                name: "first",
                type_name: "string",
            },
            Eip712Field {
                name: "second",
                type_name: "string",
            },
        ],
    }];
    let values = [
        Eip712Value {
            name: "first",
            value: Eip712ValueKind::String("abc"),
        },
        Eip712Value {
            name: "second",
            value: Eip712ValueKind::String("def"),
        },
    ];
    let mut output = [0xa5_u8; 64];
    let mut scratch = [0u8; 128];

    assert_eq!(
        encode_eip712_data_with_limits::<RealKeccak>(
            &types,
            "Payload",
            &values,
            Eip712Limits {
                max_value_nodes: EIP712_MAX_VALUE_NODES,
                max_dynamic_value_bytes: 6,
            },
            &mut output,
            &mut scratch,
        ),
        Ok(64)
    );
    output.fill(0xa5);
    assert_eq!(
        encode_eip712_data_with_limits::<RealKeccak>(
            &types,
            "Payload",
            &values,
            Eip712Limits {
                max_value_nodes: EIP712_MAX_VALUE_NODES,
                max_dynamic_value_bytes: 5,
            },
            &mut output,
            &mut scratch,
        ),
        Err(Eip712EncodeError::ResourceLimit)
    );
    assert_eq!(output, [0u8; 64]);
}
