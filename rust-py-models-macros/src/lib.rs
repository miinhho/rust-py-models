mod attrs;
mod derive;
mod fields;
mod generics;
mod model;
mod python;
mod type_expr;

use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(PY, attributes(py, serde))]
pub fn derive_py(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    match derive::expand(input) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.to_compile_error().into(),
    }
}
