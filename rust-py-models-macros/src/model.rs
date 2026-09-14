mod enumeration;
mod structure;

mod tagging;
use proc_macro2::TokenStream as Tokens;
use syn::{DataEnum, DataStruct};

pub(super) struct ModelPieces {
    pub(super) declarations: Vec<Tokens>,
    pub(super) hashable: Tokens,
}

pub(super) fn structure(
    data: DataStruct,
    name: &str,
    attr: &crate::attrs::Options,
    params: &[String],
) -> syn::Result<ModelPieces> {
    structure::render(data, name, attr, params)
}

pub(super) fn enumeration(
    data: DataEnum,
    name: &str,
    span: proc_macro2::Span,
    attr: &crate::attrs::Options,
    params: &[String],
) -> syn::Result<ModelPieces> {
    enumeration::render(data, name, span, attr, params)
}

pub(super) fn dataclass_options(options: crate::attrs::DataclassOptions) -> Tokens {
    let frozen = option_bool(options.frozen);
    let slots = option_bool(options.slots);
    let kw_only = option_bool(options.kw_only);
    quote::quote! {
        ::rust_py_models::DataclassOptions {
            frozen: #frozen,
            slots: #slots,
            kw_only: #kw_only,
        }
    }
}

fn option_bool(value: Option<bool>) -> Tokens {
    match value {
        Some(value) => quote::quote! { Some(#value) },
        None => quote::quote! { None },
    }
}

fn documentation(value: &Option<String>) -> Tokens {
    value.as_ref().map_or_else(
        || quote::quote! { None },
        |value| quote::quote! { Some(String::from(#value)) },
    )
}
