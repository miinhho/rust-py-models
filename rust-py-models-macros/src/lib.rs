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
mod model;
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
