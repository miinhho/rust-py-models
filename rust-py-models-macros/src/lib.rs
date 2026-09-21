#![warn(missing_docs)]
#![warn(rustdoc::broken_intra_doc_links)]

//! Derive implementation for `rust-py-models`.
//!
//! Applications use the re-exported `rust_py_models::PY` derive rather than
//! depending on this crate directly.

mod analysis;
mod attrs;
mod derive;
mod fields;
mod generics;
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
    let result = if let Some(decision) = analysis::decision(&input) {
        derive::expand_with(input, &decision.projection, Some(&decision.visible))
    } else if input.generics.type_params().next().is_some() {
        Err(syn::Error::new_spanned(
            &input.ident,
            "PY could not locate this generic model in the package source; declare it in a Rust module file so its dependencies can be analyzed",
        ))
    } else {
        derive::expand(input)
    };
    match result {
        Ok(tokens) => tokens.into(),
        Err(err) => err.to_compile_error().into(),
    }
}
