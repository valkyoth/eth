use super::*;
use syn::parse_quote;

#[test]
fn rlp_encode_plan_preserves_declaration_order() {
    let input: DeriveInput = parse_quote! {
        struct TransactionFields {
            chain_id: u64,
            nonce: u64,
            value: u128,
        }
    };

    let plan = plan_rlp_derive(&input, RlpDeriveKind::Encode);

    assert!(matches!(
        plan,
        Ok(RlpDerivePlan {
            kind: RlpDeriveKind::Encode,
            ref fields,
        }) if fields == &vec![
                RlpFieldPlan {
                    index: 0,
                    mode: RlpFieldMode::Required,
                    skip_reason: None,
                },
                RlpFieldPlan {
                    index: 1,
                    mode: RlpFieldMode::Required,
                    skip_reason: None,
                },
                RlpFieldPlan {
                    index: 2,
                    mode: RlpFieldMode::Required,
                    skip_reason: None,
                },
            ]
    ));
}

#[test]
fn rlp_decode_plan_requires_explicit_skip_default_reason() {
    let input: DeriveInput = parse_quote! {
        struct WithCachedField {
            payload: u64,
            #[eth_rlp(skip, default, reason = "derived cache")]
            cached_hash: [u8; 32],
        }
    };

    let plan = plan_rlp_derive(&input, RlpDeriveKind::Decode);

    assert!(matches!(
        plan,
        Ok(RlpDerivePlan {
            kind: RlpDeriveKind::Decode,
            ref fields,
        }) if fields == &vec![
                RlpFieldPlan {
                    index: 0,
                    mode: RlpFieldMode::Required,
                    skip_reason: None,
                },
                RlpFieldPlan {
                    index: 1,
                    mode: RlpFieldMode::SkipWithDefault,
                    skip_reason: Some(String::from("derived cache")),
                },
            ]
    ));
}

#[test]
fn rlp_derives_reject_generics_until_bounds_are_designed() {
    let input: DeriveInput = parse_quote! {
        struct Wrapper<T> {
            value: T,
        }
    };

    let result = plan_rlp_derive(&input, RlpDeriveKind::Encode);

    assert!(matches!(
        result,
        Err(error) if error.to_string().contains("does not support generics")
    ));
}

#[test]
fn rlp_derives_reject_enums_and_unions() {
    let enum_input: DeriveInput = parse_quote! {
        enum Choice {
            A(u64),
            B(u64),
        }
    };
    let union_input: DeriveInput = parse_quote! {
        union Choice {
            a: u64,
            b: u64,
        }
    };

    let enum_result = plan_rlp_derive(&enum_input, RlpDeriveKind::Decode);
    let union_result = plan_rlp_derive(&union_input, RlpDeriveKind::Decode);

    assert!(matches!(
        enum_result,
        Err(error) if error.to_string().contains("not available for enums")
    ));
    assert!(matches!(
        union_result,
        Err(error) if error.to_string().contains("not available for unions")
    ));
}

#[test]
fn rlp_field_attrs_reject_ambiguous_skip_or_default() {
    let skip_only: syn::Field = parse_quote! {
        #[eth_rlp(skip)]
        cached_hash: [u8; 32]
    };
    let default_only: syn::Field = parse_quote! {
        #[eth_rlp(default)]
        cached_hash: [u8; 32]
    };
    let reason_only: syn::Field = parse_quote! {
        #[eth_rlp(reason = "derived cache")]
        cached_hash: [u8; 32]
    };

    assert!(matches!(
        rlp_field_attrs(&skip_only),
        Err(error) if error.to_string().contains("skip requires default")
    ));
    assert!(matches!(
        rlp_field_attrs(&default_only),
        Err(error) if error.to_string().contains("default is only supported")
    ));
    assert!(matches!(
        rlp_field_attrs(&reason_only),
        Err(error) if error.to_string().contains("reason is only supported")
    ));
}

#[test]
fn rlp_field_attrs_reject_duplicate_attributes() {
    let field: syn::Field = parse_quote! {
        #[eth_rlp(skip, default, reason = "derived cache")]
        #[eth_rlp(reason = "conflicting reason")]
        cached_hash: [u8; 32]
    };

    let result = rlp_field_attrs(&field);

    assert!(matches!(result, Err(error) if error.to_string().contains("duplicate")));
}
