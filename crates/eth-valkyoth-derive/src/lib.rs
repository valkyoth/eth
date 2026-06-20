#![forbid(unsafe_code)]
//! Optional derive macros for `eth-valkyoth-sanitization`.

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote};
use syn::{
    Data, DeriveInput, Error, Fields, Generics, LitStr, Path, parse_macro_input, parse_quote,
};

/// Derives `eth_valkyoth_sanitization::SecureSanitize` for structs and enums.
///
/// Fields marked `#[eth_sanitization(skip)]` are intentionally not sanitized.
/// Use skips only for fields that cannot carry secret material.
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
    let crate_path = crate_path(input)?;
    let name = &input.ident;
    let mut generics = input.generics.clone();
    add_sanitize_bounds(&mut generics, &crate_path);
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
    let body = sanitize_body(&input.data, &crate_path)?;

    Ok(quote! {
        impl #impl_generics #crate_path::SecureSanitize for #name #ty_generics #where_clause {
            fn secure_sanitize(&mut self) {
                #body
            }
        }
    })
}

fn expand_secure_sanitize_on_drop(input: &DeriveInput) -> Result<TokenStream2, Error> {
    let crate_path = crate_path(input)?;
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

fn add_sanitize_bounds(generics: &mut Generics, crate_path: &Path) {
    for param in generics.type_params_mut() {
        let bound = parse_quote!(#crate_path::SecureSanitize);
        param.bounds.push(bound);
    }
}

fn sanitize_body(data: &Data, crate_path: &Path) -> Result<TokenStream2, Error> {
    match data {
        Data::Struct(data) => sanitize_struct_fields(&data.fields, crate_path),
        Data::Enum(data) => {
            let arms = data
                .variants
                .iter()
                .map(|variant| sanitize_variant(variant, crate_path))
                .collect::<Result<Vec<_>, _>>()?;
            Ok(quote! {
                match self {
                    #(#arms),*
                }
            })
        }
        Data::Union(data) => Err(Error::new_spanned(
            data.union_token,
            "SecureSanitize cannot be derived for unions",
        )),
    }
}

fn sanitize_struct_fields(fields: &Fields, crate_path: &Path) -> Result<TokenStream2, Error> {
    let calls = field_accesses(fields)?
        .into_iter()
        .filter(|field| !field.skip)
        .map(|field| {
            let access = field.access;
            quote!(#crate_path::SecureSanitize::secure_sanitize(&mut self.#access);)
        });
    Ok(quote!(#(#calls)*))
}

fn sanitize_variant(variant: &syn::Variant, crate_path: &Path) -> Result<TokenStream2, Error> {
    let name = &variant.ident;
    match &variant.fields {
        Fields::Named(fields) => {
            let bindings = fields.named.iter().map(|field| field.ident.as_ref());
            let mut calls = Vec::new();
            for field in &fields.named {
                if !has_skip(field)? {
                    let ident = field.ident.as_ref();
                    calls.push(quote!(#crate_path::SecureSanitize::secure_sanitize(#ident);));
                }
            }
            Ok(quote!(Self::#name { #(#bindings),* } => { #(#calls)* }))
        }
        Fields::Unnamed(fields) => {
            let bindings = (0..fields.unnamed.len())
                .map(|index| format_ident!("field_{index}"))
                .collect::<Vec<_>>();
            let mut calls = Vec::new();
            for (field, binding) in fields.unnamed.iter().zip(bindings.iter()) {
                if !has_skip(field)? {
                    calls.push(quote!(#crate_path::SecureSanitize::secure_sanitize(#binding);));
                }
            }
            Ok(quote!(Self::#name(#(#bindings),*) => { #(#calls)* }))
        }
        Fields::Unit => Ok(quote!(Self::#name => {})),
    }
}

struct FieldAccess {
    access: TokenStream2,
    skip: bool,
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
                    skip: has_skip(field)?,
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
                    skip: has_skip(field)?,
                })
            })
            .collect(),
        Fields::Unit => Ok(Vec::new()),
    }
}

fn crate_path(input: &DeriveInput) -> Result<Path, Error> {
    let mut path = parse_quote!(::eth_valkyoth_sanitization);
    for attr in input
        .attrs
        .iter()
        .filter(|attr| attr.path().is_ident("eth_sanitization"))
    {
        attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("crate") {
                let value = meta.value()?;
                let literal: LitStr = value.parse()?;
                path = literal.parse()?;
                Ok(())
            } else {
                Err(meta.error("unsupported eth_sanitization container attribute"))
            }
        })?;
    }
    Ok(path)
}

fn has_skip(field: &syn::Field) -> Result<bool, Error> {
    let mut skip = false;
    for attr in field
        .attrs
        .iter()
        .filter(|attr| attr.path().is_ident("eth_sanitization"))
    {
        attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("skip") {
                skip = true;
                Ok(())
            } else {
                Err(meta.error("unsupported eth_sanitization field attribute"))
            }
        })?;
    }
    Ok(skip)
}
