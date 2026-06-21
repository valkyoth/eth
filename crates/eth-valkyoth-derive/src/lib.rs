#![forbid(unsafe_code)]
//! Optional derive macros for `eth-valkyoth-sanitization`.

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{
    Data, DeriveInput, Error, Fields, GenericArgument, Generics, Ident, LitStr, Path,
    PathArguments, Type, parse_macro_input, parse_quote,
};

/// Derives `eth_valkyoth_sanitization::SecureSanitize` for structs.
///
/// Fields marked `#[eth_sanitization(skip, reason = "...")]` are intentionally
/// not sanitized. Use skips only for fields that cannot carry secret material.
///
/// Enums are rejected because Rust does not clear inactive variant backing
/// bytes when the active variant changes. Use a struct wrapper for secret
/// material until a verified full-width clear primitive is available.
#[proc_macro_derive(SecureSanitize, attributes(eth_sanitization))]
pub fn derive_secure_sanitize(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    expand_secure_sanitize(&input)
        .unwrap_or_else(Error::into_compile_error)
        .into()
}

/// Derives `Drop` by calling `SecureSanitize::secure_sanitize`.
///
/// This macro expects the type to implement `SecureSanitize`, normally by also
/// deriving `SecureSanitize`.
#[proc_macro_derive(SecureSanitizeOnDrop, attributes(eth_sanitization))]
pub fn derive_secure_sanitize_on_drop(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    expand_secure_sanitize_on_drop(&input)
        .unwrap_or_else(Error::into_compile_error)
        .into()
}

fn expand_secure_sanitize(input: &DeriveInput) -> Result<TokenStream2, Error> {
    let attrs = container_attrs(input)?;
    let crate_path = attrs.crate_path;
    let name = &input.ident;
    let body = sanitize_body(&input.data, &crate_path)?;
    let mut generics = input.generics.clone();
    let secret_params = sanitized_type_params(&input.data, input.generics.type_params())?;
    add_sanitize_bounds(&mut generics, &crate_path, &secret_params);
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    Ok(quote! {
        impl #impl_generics #crate_path::SecureSanitize for #name #ty_generics #where_clause {
            fn secure_sanitize(&mut self) {
                #body
            }
        }
    })
}

fn expand_secure_sanitize_on_drop(input: &DeriveInput) -> Result<TokenStream2, Error> {
    let crate_path = container_attrs(input)?.crate_path;
    match &input.data {
        Data::Struct(_) => {}
        Data::Enum(data) => {
            return Err(Error::new_spanned(
                data.enum_token,
                "SecureSanitizeOnDrop cannot be derived for enums; use a struct wrapper",
            ));
        }
        Data::Union(data) => {
            return Err(Error::new_spanned(
                data.union_token,
                "SecureSanitizeOnDrop cannot be derived for unions; use a struct wrapper",
            ));
        }
    }
    let name = &input.ident;
    let generics = input.generics.clone();
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    Ok(quote! {
        impl #impl_generics Drop for #name #ty_generics #where_clause {
            fn drop(&mut self) {
                #crate_path::SecureSanitize::secure_sanitize(self);
            }
        }
    })
}

fn add_sanitize_bounds(generics: &mut Generics, crate_path: &Path, secret_params: &[Ident]) {
    for param in generics.type_params_mut() {
        if secret_params.iter().any(|ident| ident == &param.ident) {
            let bound = parse_quote!(#crate_path::SecureSanitize);
            param.bounds.push(bound);
        }
    }
}

fn sanitize_body(data: &Data, crate_path: &Path) -> Result<TokenStream2, Error> {
    match data {
        Data::Struct(data) => sanitize_struct_fields(&data.fields, crate_path),
        Data::Enum(data) => Err(Error::new_spanned(
            data.enum_token,
            "SecureSanitize cannot be derived for enums because inactive variant bytes may retain secrets; use a struct wrapper",
        )),
        Data::Union(data) => Err(Error::new_spanned(
            data.union_token,
            "SecureSanitize cannot be derived for unions",
        )),
    }
}

fn sanitize_struct_fields(fields: &Fields, crate_path: &Path) -> Result<TokenStream2, Error> {
    let calls = field_accesses(fields)?
        .into_iter()
        .filter(|field| field.skip_reason.is_none())
        .map(|field| {
            let access = field.access;
            quote!(#crate_path::SecureSanitize::secure_sanitize(&mut self.#access);)
        });
    Ok(quote!(#(#calls)*))
}

struct FieldAccess {
    access: TokenStream2,
    skip_reason: Option<LitStr>,
}

fn field_accesses(fields: &Fields) -> Result<Vec<FieldAccess>, Error> {
    match fields {
        Fields::Named(fields) => fields
            .named
            .iter()
            .map(|field| {
                let ident = field
                    .ident
                    .as_ref()
                    .ok_or_else(|| Error::new_spanned(field, "missing field ident"))?;
                Ok(FieldAccess {
                    access: quote!(#ident),
                    skip_reason: skip_reason(field)?,
                })
            })
            .collect(),
        Fields::Unnamed(fields) => fields
            .unnamed
            .iter()
            .enumerate()
            .map(|(index, field)| {
                let access = syn::Index::from(index);
                Ok(FieldAccess {
                    access: quote!(#access),
                    skip_reason: skip_reason(field)?,
                })
            })
            .collect(),
        Fields::Unit => Ok(Vec::new()),
    }
}

struct ContainerAttrs {
    crate_path: Path,
}

fn container_attrs(input: &DeriveInput) -> Result<ContainerAttrs, Error> {
    let mut attrs = ContainerAttrs {
        crate_path: parse_quote!(::eth_valkyoth_sanitization),
    };
    for attr in input
        .attrs
        .iter()
        .filter(|attr| attr.path().is_ident("eth_sanitization"))
    {
        attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("crate") {
                let value = meta.value()?;
                let literal: LitStr = value.parse()?;
                attrs.crate_path = literal.parse()?;
                Ok(())
            } else {
                Err(meta.error("unsupported eth_sanitization container attribute"))
            }
        })?;
    }
    Ok(attrs)
}

fn skip_reason(field: &syn::Field) -> Result<Option<LitStr>, Error> {
    let mut skip = false;
    let mut reason = None;
    for attr in field
        .attrs
        .iter()
        .filter(|attr| attr.path().is_ident("eth_sanitization"))
    {
        attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("skip") {
                skip = true;
                Ok(())
            } else if meta.path.is_ident("reason") {
                let value = meta.value()?;
                let literal: LitStr = value.parse()?;
                if literal.value().trim().is_empty() {
                    Err(meta.error("eth_sanitization skip reason must not be empty"))
                } else {
                    reason = Some(literal);
                    Ok(())
                }
            } else {
                Err(meta.error("unsupported eth_sanitization field attribute"))
            }
        })?;
    }
    if skip && reason.is_none() {
        return Err(Error::new_spanned(
            field,
            "eth_sanitization skip requires reason = \"...\" acknowledging the field is non-secret",
        ));
    }
    if !skip && reason.is_some() {
        return Err(Error::new_spanned(
            field,
            "eth_sanitization reason is only supported together with skip",
        ));
    }
    Ok(reason)
}

fn sanitized_type_params<'a>(
    data: &Data,
    type_params: impl Iterator<Item = &'a syn::TypeParam>,
) -> Result<Vec<Ident>, Error> {
    let params = type_params
        .map(|param| param.ident.clone())
        .collect::<Vec<_>>();
    let fields = match data {
        Data::Struct(data) => &data.fields,
        Data::Enum(_) | Data::Union(_) => return Ok(Vec::new()),
    };
    let mut used = Vec::new();
    for field in iter_fields(fields) {
        if skip_reason(field)?.is_some() {
            continue;
        }
        for ident in &params {
            if type_uses_ident(&field.ty, ident) && !used.iter().any(|used| used == ident) {
                used.push(ident.clone());
            }
        }
    }
    Ok(used)
}

fn iter_fields(fields: &Fields) -> Vec<&syn::Field> {
    match fields {
        Fields::Named(fields) => fields.named.iter().collect(),
        Fields::Unnamed(fields) => fields.unnamed.iter().collect(),
        Fields::Unit => Vec::new(),
    }
}

fn type_uses_ident(ty: &Type, ident: &Ident) -> bool {
    match ty {
        Type::Array(ty) => type_uses_ident(&ty.elem, ident),
        Type::BareFn(ty) => {
            ty.inputs
                .iter()
                .any(|input| type_uses_ident(&input.ty, ident))
                || matches!(&ty.output, syn::ReturnType::Type(_, ty) if type_uses_ident(ty, ident))
        }
        Type::Group(ty) => type_uses_ident(&ty.elem, ident),
        Type::Paren(ty) => type_uses_ident(&ty.elem, ident),
        Type::Path(ty) => path_uses_ident(&ty.path, ident),
        Type::Ptr(ty) => type_uses_ident(&ty.elem, ident),
        Type::Reference(ty) => type_uses_ident(&ty.elem, ident),
        Type::Slice(ty) => type_uses_ident(&ty.elem, ident),
        Type::Tuple(ty) => ty.elems.iter().any(|elem| type_uses_ident(elem, ident)),
        // Conservative fallback: a spurious bound becomes a compile error, but
        // a missing bound can silently skip sanitization of secret material.
        _ => true,
    }
}

fn path_uses_ident(path: &Path, ident: &Ident) -> bool {
    path.segments.iter().any(|segment| {
        segment.ident == *ident
            || match &segment.arguments {
                PathArguments::AngleBracketed(arguments) => arguments.args.iter().any(|arg| {
                    matches!(arg, GenericArgument::Type(ty) if type_uses_ident(ty, ident))
                }),
                PathArguments::Parenthesized(arguments) => {
                    arguments.inputs.iter().any(|ty| type_uses_ident(ty, ident))
                        || matches!(&arguments.output, syn::ReturnType::Type(_, ty) if type_uses_ident(ty, ident))
                }
                PathArguments::None => false,
            }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::parse_quote;

    #[test]
    fn skip_requires_reason() {
        let field: syn::Field = parse_quote! {
            #[eth_sanitization(skip)]
            label: u8
        };

        let result = skip_reason(&field);

        assert!(matches!(result, Err(error) if error.to_string().contains("skip requires reason")));
    }

    #[test]
    fn skip_accepts_non_empty_reason() {
        let field: syn::Field = parse_quote! {
            #[eth_sanitization(skip, reason = "non-secret label")]
            label: u8
        };

        let reason = skip_reason(&field);

        assert!(matches!(reason, Ok(Some(reason)) if reason.value() == "non-secret label"));
    }

    #[test]
    fn enum_derives_are_rejected() {
        let input: DeriveInput = parse_quote! {
            enum SecretChoice {
                Key(u8),
                Empty,
            }
        };

        let result = expand_secure_sanitize(&input);

        assert!(
            matches!(result, Err(error) if error.to_string().contains("cannot be derived for enums"))
        );
    }

    #[test]
    fn enum_drop_derives_are_rejected() {
        let input: DeriveInput = parse_quote! {
            enum SecretChoice {
                Key(u8),
                Empty,
            }
        };

        let result = expand_secure_sanitize_on_drop(&input);

        assert!(
            matches!(result, Err(error) if error.to_string().contains("SecureSanitizeOnDrop cannot be derived for enums"))
        );
    }

    #[test]
    fn union_drop_derives_are_rejected() {
        let input: DeriveInput = parse_quote! {
            union SecretChoice {
                key: [u8; 32],
                flag: u8,
            }
        };

        let result = expand_secure_sanitize_on_drop(&input);

        assert!(
            matches!(result, Err(error) if error.to_string().contains("SecureSanitizeOnDrop cannot be derived for unions"))
        );
    }

    #[test]
    fn skipped_generic_fields_do_not_receive_sanitize_bounds() {
        let input: DeriveInput = parse_quote! {
            struct Wrapper<T, Label> {
                secret: T,
                #[eth_sanitization(skip, reason = "non-secret label")]
                label: Label,
            }
        };

        let result = expand_secure_sanitize(&input);
        assert!(result.is_ok());
        let rendered =
            result.map_or_else(|_| std::string::String::new(), |output| output.to_string());

        assert!(rendered.contains("T :"));
        assert!(!rendered.contains("Label :"));
    }
}
