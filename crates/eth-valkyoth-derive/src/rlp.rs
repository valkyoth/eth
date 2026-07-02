use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote};
use syn::{
    Data, DeriveInput, Error, Fields, Generics, LitStr, Path, parse_quote, spanned::Spanned,
};

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
    skip_reason: Option<String>,
}

#[derive(Debug, Eq, PartialEq)]
struct RlpFieldAttrs {
    mode: RlpFieldMode,
    skip_reason: Option<String>,
}

#[derive(Debug, Eq, PartialEq)]
struct RlpDerivePlan {
    kind: RlpDeriveKind,
    fields: Vec<RlpFieldPlan>,
}

struct RlpContainerAttrs {
    crate_path: Path,
}

pub(crate) fn expand_rlp_encode(input: &DeriveInput) -> Result<TokenStream2, Error> {
    let attrs = rlp_container_attrs(input)?;
    let crate_path = attrs.crate_path;
    let plan = plan_rlp_derive(input, RlpDeriveKind::Encode)?;
    let name = &input.ident;
    let required_count = required_field_count(&plan);
    let fields = struct_fields(input)?;
    let len_steps = encode_len_steps(fields, &plan, &crate_path)?;
    let write_steps = encode_write_steps(fields, &plan, &crate_path)?;

    Ok(quote! {
        impl #crate_path::RlpEncode for #name {
            type Error = #crate_path::RlpDeriveError;

            fn encoded_rlp_len(&self) -> ::core::result::Result<usize, Self::Error> {
                let mut payload_len = 0usize;
                #(#len_steps)*
                let header_len = #crate_path::encoded_rlp_list_header_len(payload_len)
                    .map_err(#crate_path::RlpDeriveError::from)?;
                #crate_path::checked_encoded_len_add(header_len, payload_len)
            }

            fn encode_rlp(
                &self,
                output: &mut [u8],
            ) -> ::core::result::Result<usize, Self::Error> {
                let mut payload_len = 0usize;
                #(#len_steps)*
                let header_len = #crate_path::encoded_rlp_list_header_len(payload_len)
                    .map_err(#crate_path::RlpDeriveError::from)?;
                let total_len = #crate_path::checked_encoded_len_add(header_len, payload_len)?;
                if output.len() < total_len {
                    return Err(#crate_path::RlpDeriveError::Decode(
                        #crate_path::DecodeError::OffsetOutOfBounds,
                    ));
                }
                let mut cursor = #crate_path::encode_rlp_list_header(payload_len, output)
                    .map_err(#crate_path::RlpDeriveError::from)?;
                #(#write_steps)*
                debug_assert_eq!(cursor, total_len);
                let _eth_rlp_required_field_count: usize = #required_count;
                Ok(total_len)
            }
        }
    })
}

pub(crate) fn expand_rlp_decode(input: &DeriveInput) -> Result<TokenStream2, Error> {
    let attrs = rlp_container_attrs(input)?;
    let crate_path = attrs.crate_path;
    let plan = plan_rlp_derive(input, RlpDeriveKind::Decode)?;
    let name = &input.ident;
    let required_count = required_field_count(&plan);
    let fields = struct_fields(input)?;
    let decode_steps = decode_steps(fields, &plan, &crate_path)?;
    let construction = construct_value(name, fields, &plan)?;

    Ok(quote! {
        impl #crate_path::RlpDecode for #name {
            type Error = #crate_path::RlpDeriveError;

            fn decode_rlp(
                input: &[u8],
                limits: #crate_path::DecodeLimits,
            ) -> ::core::result::Result<Self, Self::Error> {
                let list = #crate_path::decode_rlp_list(input, limits)
                    .map_err(#crate_path::RlpDeriveError::from)?;
                if list.item_count() != #required_count {
                    return Err(#crate_path::RlpDeriveError::WrongFieldCount {
                        expected: #required_count,
                        found: list.item_count(),
                    });
                }
                let mut items = list.items();
                #(#decode_steps)*
                Ok(#construction)
            }

            fn decode_rlp_item(
                item: #crate_path::RlpItem<'_>,
            ) -> ::core::result::Result<Self, Self::Error> {
                let list = item.as_list().ok_or(#crate_path::RlpDeriveError::Decode(
                    #crate_path::DecodeError::UnexpectedScalar,
                ))?;
                if list.item_count() != #required_count {
                    return Err(#crate_path::RlpDeriveError::WrongFieldCount {
                        expected: #required_count,
                        found: list.item_count(),
                    });
                }
                let mut items = list.items();
                #(#decode_steps)*
                Ok(#construction)
            }
        }
    })
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

fn rlp_container_attrs(input: &DeriveInput) -> Result<RlpContainerAttrs, Error> {
    let mut attrs = RlpContainerAttrs {
        crate_path: parse_quote!(::eth_valkyoth_codec),
    };
    for attr in input
        .attrs
        .iter()
        .filter(|attr| attr.path().is_ident("eth_rlp"))
    {
        attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("crate") {
                let value = meta.value()?;
                let literal: LitStr = value.parse()?;
                attrs.crate_path = literal.parse()?;
                Ok(())
            } else {
                Err(meta.error("unsupported eth_rlp container attribute"))
            }
        })?;
    }
    Ok(attrs)
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
                let attrs = rlp_field_attrs(field)?;
                Ok(RlpFieldPlan {
                    index,
                    mode: attrs.mode,
                    skip_reason: attrs.skip_reason,
                })
            })
            .collect(),
        Fields::Unnamed(fields) => fields
            .unnamed
            .iter()
            .enumerate()
            .map(|(index, field)| {
                let attrs = rlp_field_attrs(field)?;
                Ok(RlpFieldPlan {
                    index,
                    mode: attrs.mode,
                    skip_reason: attrs.skip_reason,
                })
            })
            .collect(),
        Fields::Unit => Ok(Vec::new()),
    }
}

fn required_field_count(plan: &RlpDerivePlan) -> usize {
    plan.fields
        .iter()
        .filter(|field| matches!(field.mode, RlpFieldMode::Required))
        .count()
}

fn struct_fields(input: &DeriveInput) -> Result<&Fields, Error> {
    match &input.data {
        Data::Struct(data) => Ok(&data.fields),
        Data::Enum(data) => Err(Error::new_spanned(
            data.enum_token,
            "RLP derives are not available for enums; use an explicit hand-written domain decoder",
        )),
        Data::Union(data) => Err(Error::new_spanned(
            data.union_token,
            "RLP derives are not available for unions",
        )),
    }
}

fn encode_len_steps(
    fields: &Fields,
    plan: &RlpDerivePlan,
    crate_path: &Path,
) -> Result<Vec<TokenStream2>, Error> {
    field_tokens(fields, plan, |field, access, _binding, mode| match mode {
        RlpFieldMode::Required => Ok(quote! {
            payload_len = #crate_path::checked_encoded_len_add(
                payload_len,
                #crate_path::RlpEncode::encoded_rlp_len(&self.#access)
                    .map_err(#crate_path::RlpDeriveError::from)?,
            )?;
        }),
        RlpFieldMode::SkipWithDefault => {
            let reason = skip_reason_literal(field)?;
            Ok(quote! {
                let _eth_rlp_skip_reason: &str = #reason;
            })
        }
    })
}

fn encode_write_steps(
    fields: &Fields,
    plan: &RlpDerivePlan,
    crate_path: &Path,
) -> Result<Vec<TokenStream2>, Error> {
    field_tokens(fields, plan, |field, access, _binding, mode| match mode {
        RlpFieldMode::Required => Ok(quote! {
            let target = output.get_mut(cursor..total_len).ok_or(
                #crate_path::RlpDeriveError::Decode(#crate_path::DecodeError::OffsetOutOfBounds),
            )?;
            let written = #crate_path::RlpEncode::encode_rlp(&self.#access, target)
                .map_err(#crate_path::RlpDeriveError::from)?;
            cursor = #crate_path::checked_encoded_len_add(cursor, written)?;
        }),
        RlpFieldMode::SkipWithDefault => {
            let reason = skip_reason_literal(field)?;
            Ok(quote! {
                let _eth_rlp_skip_reason: &str = #reason;
            })
        }
    })
}

fn decode_steps(
    fields: &Fields,
    plan: &RlpDerivePlan,
    crate_path: &Path,
) -> Result<Vec<TokenStream2>, Error> {
    let expected_count = required_field_count(plan);
    field_tokens(fields, plan, |field, _access, binding, mode| {
        let ty = &field.ty;
        match mode {
            RlpFieldMode::Required => Ok(quote! {
                let item = items.next().ok_or(#crate_path::RlpDeriveError::WrongFieldCount {
                    expected: #expected_count,
                    found: #expected_count,
                })?;
                let item = item.map_err(#crate_path::RlpDeriveError::from)?;
                let #binding = <#ty as #crate_path::RlpDecode>::decode_rlp_item(item)
                    .map_err(#crate_path::RlpDeriveError::from)?;
            }),
            RlpFieldMode::SkipWithDefault => {
                let reason = skip_reason_literal(field)?;
                Ok(quote! {
                    let _eth_rlp_skip_reason: &str = #reason;
                    let #binding = <#ty as ::core::default::Default>::default();
                })
            }
        }
    })
}

fn construct_value(
    name: &syn::Ident,
    fields: &Fields,
    _plan: &RlpDerivePlan,
) -> Result<TokenStream2, Error> {
    match fields {
        Fields::Named(fields) => {
            let entries = fields.named.iter().enumerate().map(|(index, field)| {
                let ident = field.ident.as_ref().ok_or_else(|| {
                    Error::new_spanned(field, "RLP derive expected a named field")
                })?;
                let binding = binding_ident(index);
                Ok::<TokenStream2, Error>(quote!(#ident: #binding))
            });
            let entries = entries.collect::<Result<Vec<_>, _>>()?;
            Ok(quote!(#name { #(#entries),* }))
        }
        Fields::Unnamed(fields) => {
            let entries = fields
                .unnamed
                .iter()
                .enumerate()
                .map(|(index, _field)| binding_ident(index));
            Ok(quote!(#name(#(#entries),*)))
        }
        Fields::Unit => Ok(quote!(#name)),
    }
}

fn field_tokens(
    fields: &Fields,
    plan: &RlpDerivePlan,
    mut build: impl FnMut(
        &syn::Field,
        TokenStream2,
        syn::Ident,
        RlpFieldMode,
    ) -> Result<TokenStream2, Error>,
) -> Result<Vec<TokenStream2>, Error> {
    let field_refs = match fields {
        Fields::Named(fields) => fields.named.iter().collect::<Vec<_>>(),
        Fields::Unnamed(fields) => fields.unnamed.iter().collect::<Vec<_>>(),
        Fields::Unit => Vec::new(),
    };
    field_refs
        .into_iter()
        .zip(plan.fields.iter())
        .map(|(field, planned)| {
            let access = field_access(fields, field, planned.index)?;
            let binding = binding_ident(planned.index);
            build(field, access, binding, planned.mode)
        })
        .collect()
}

fn field_access(fields: &Fields, field: &syn::Field, index: usize) -> Result<TokenStream2, Error> {
    match fields {
        Fields::Named(_) => {
            let ident = field
                .ident
                .as_ref()
                .ok_or_else(|| Error::new_spanned(field, "RLP derive expected named field"))?;
            Ok(quote!(#ident))
        }
        Fields::Unnamed(_) => {
            let access = syn::Index::from(index);
            Ok(quote!(#access))
        }
        Fields::Unit => Err(Error::new_spanned(field, "RLP derive received no field")),
    }
}

fn binding_ident(index: usize) -> syn::Ident {
    format_ident!("eth_rlp_field_{index}")
}

fn skip_reason_literal(field: &syn::Field) -> Result<LitStr, Error> {
    let reason = rlp_field_attrs(field)?.skip_reason.ok_or_else(|| {
        Error::new_spanned(field, "RLP skipped field must retain a reason literal")
    })?;
    Ok(LitStr::new(&reason, field.span()))
}

fn rlp_field_attrs(field: &syn::Field) -> Result<RlpFieldAttrs, Error> {
    let mut skip = false;
    let mut default = false;
    let mut reason = None::<LitStr>;
    let mut seen_attr = false;
    for attr in field
        .attrs
        .iter()
        .filter(|attr| attr.path().is_ident("eth_rlp"))
    {
        if seen_attr {
            return Err(Error::new_spanned(
                attr,
                "duplicate #[eth_rlp(...)] attribute on field",
            ));
        }
        seen_attr = true;
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
        (false, false, false) => Ok(RlpFieldAttrs {
            mode: RlpFieldMode::Required,
            skip_reason: None,
        }),
        (true, true, true) => Ok(RlpFieldAttrs {
            mode: RlpFieldMode::SkipWithDefault,
            skip_reason: reason.map(|reason| reason.value()),
        }),
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
#[path = "rlp_tests.rs"]
mod tests;
