#![warn(missing_docs)]
#![warn(rustdoc::broken_intra_doc_links)]

//! Derive implementation for `rust-py-models`.
//!
//! Applications use the re-exported `rust_py_models::PY` derive rather than
//! depending on this crate directly.

mod attrs;
mod derive;
mod fields;
mod generics;
mod group;
mod model;
mod projection;
mod python;
mod type_expr;

use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput};

/// Derives the Python model contract for a Rust struct or enum.
#[proc_macro_derive(PY, attributes(py, serde))]
pub fn derive_py(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    match derive::expand(input) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

/// Resolves Python-visible generic parameters across models in an inline module.
///
/// Use this when a model forwards a type parameter through another derived
/// model, especially when models refer to each other. Every participating
/// struct or enum must be directly inside the inline module and derive `PY`.
/// Parameters of types outside the group are treated as visible unless a
/// built-in mapping explicitly ignores them.
///
/// # Errors
///
/// Rejects arguments and out-of-line modules. Derive errors in grouped models
/// are reported at the corresponding model or field.
#[proc_macro_attribute]
pub fn py_models(args: TokenStream, input: TokenStream) -> TokenStream {
    if !args.is_empty() {
        return syn::Error::new(
            proc_macro2::Span::call_site(),
            "#[py_models] takes no arguments",
        )
        .to_compile_error()
        .into();
    }
    let input = parse_macro_input!(input as syn::ItemMod);
    match group::expand(input) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.to_compile_error().into(),
    }
}
