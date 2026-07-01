use syn::{Data, DeriveInput, Error, Fields, Generics, LitStr};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum RlpDeriveKind {
    Encode,
    Decode,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum RlpFieldMode {
    Required,
    SkipWithDefault,
}

#[derive(Debug, Eq, PartialEq)]
struct RlpFieldPlan {
    index: usize,
    mode: RlpFieldMode,
}

#[derive(Debug, Eq, PartialEq)]
struct RlpDerivePlan {
    kind: RlpDeriveKind,
    fields: Vec<RlpFieldPlan>,
}

fn plan_rlp_derive(input: &DeriveInput, kind: RlpDeriveKind) -> Result<RlpDerivePlan, Error> {
    reject_generics(&input.generics)?;
    let fields = match &input.data {
        Data::Struct(data) => ordered_fields(&data.fields)?,
        Data::Enum(data) => {
            return Err(Error::new_spanned(
                data.enum_token,
                "RLP derives are not available for enums; use an explicit hand-written domain decoder",
            ));
        }
        Data::Union(data) => {
            return Err(Error::new_spanned(
                data.union_token,
                "RLP derives are not available for unions",
            ));
        }
    };
    Ok(RlpDerivePlan { kind, fields })
}

fn reject_generics(generics: &Generics) -> Result<(), Error> {
    if generics.params.is_empty() && generics.where_clause.is_none() {
        return Ok(());
    }
    Err(Error::new_spanned(
        generics,
        "RLP derive prototype does not support generics until bounds and lifetimes are specified",
    ))
}

fn ordered_fields(fields: &Fields) -> Result<Vec<RlpFieldPlan>, Error> {
    match fields {
        Fields::Named(fields) => fields
            .named
            .iter()
            .enumerate()
            .map(|(index, field)| {
                Ok(RlpFieldPlan {
                    index,
                    mode: rlp_field_mode(field)?,
                })
            })
            .collect(),
        Fields::Unnamed(fields) => fields
            .unnamed
            .iter()
            .enumerate()
            .map(|(index, field)| {
                Ok(RlpFieldPlan {
                    index,
                    mode: rlp_field_mode(field)?,
                })
            })
            .collect(),
        Fields::Unit => Ok(Vec::new()),
    }
}

fn rlp_field_mode(field: &syn::Field) -> Result<RlpFieldMode, Error> {
    let mut skip = false;
    let mut default = false;
    let mut reason = None::<LitStr>;
    for attr in field
        .attrs
        .iter()
        .filter(|attr| attr.path().is_ident("eth_rlp"))
    {
        attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("skip") {
                skip = true;
                Ok(())
            } else if meta.path.is_ident("default") {
                default = true;
                Ok(())
            } else if meta.path.is_ident("reason") {
                let value = meta.value()?;
                let literal: LitStr = value.parse()?;
                if literal.value().trim().is_empty() {
                    Err(meta.error("eth_rlp reason must not be empty"))
                } else {
                    reason = Some(literal);
                    Ok(())
                }
            } else {
                Err(meta.error("unsupported eth_rlp field attribute"))
            }
        })?;
    }
    match (skip, default, reason.is_some()) {
        (false, false, false) => Ok(RlpFieldMode::Required),
        (true, true, true) => Ok(RlpFieldMode::SkipWithDefault),
        (true, false, _) => Err(Error::new_spanned(
            field,
            "eth_rlp skip requires default and reason = \"...\"",
        )),
        (false, true, _) => Err(Error::new_spanned(
            field,
            "eth_rlp default is only supported together with skip",
        )),
        (false, false, true) => Err(Error::new_spanned(
            field,
            "eth_rlp reason is only supported together with skip and default",
        )),
        (true, true, false) => Err(Error::new_spanned(
            field,
            "eth_rlp skip/default requires reason = \"...\"",
        )),
    }
}

#[cfg(test)]
mod tests {
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
                        mode: RlpFieldMode::Required
                    },
                    RlpFieldPlan {
                        index: 1,
                        mode: RlpFieldMode::Required
                    },
                    RlpFieldPlan {
                        index: 2,
                        mode: RlpFieldMode::Required
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
                        mode: RlpFieldMode::Required
                    },
                    RlpFieldPlan {
                        index: 1,
                        mode: RlpFieldMode::SkipWithDefault
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

        assert!(
            matches!(result, Err(error) if error.to_string().contains("does not support generics"))
        );
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

        assert!(
            matches!(enum_result, Err(error) if error.to_string().contains("not available for enums"))
        );
        assert!(
            matches!(union_result, Err(error) if error.to_string().contains("not available for unions"))
        );
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

        assert!(
            matches!(rlp_field_mode(&skip_only), Err(error) if error.to_string().contains("skip requires default"))
        );
        assert!(
            matches!(rlp_field_mode(&default_only), Err(error) if error.to_string().contains("default is only supported"))
        );
        assert!(
            matches!(rlp_field_mode(&reason_only), Err(error) if error.to_string().contains("reason is only supported"))
        );
    }
}
