mod enumeration;
mod structure;

use proc_macro2::TokenStream as Tokens;
use syn::{DataEnum, DataStruct};

pub(super) struct ModelPieces {
    pub(super) prelude: Tokens,
    pub(super) declaration: Tokens,
    pub(super) deps: Vec<Tokens>,
    pub(super) imports: Vec<Tokens>,
    pub(super) definitions: Vec<Tokens>,
    pub(super) validations: Vec<Tokens>,
    pub(super) hashable: Tokens,
    pub(super) local_names: Vec<String>,
}

pub(super) fn structure(
    data: DataStruct,
    name: &str,
    dataclass: crate::attrs::DataclassOptions,
    params: &[String],
) -> syn::Result<ModelPieces> {
    structure::render(data, name, dataclass, params)
}

pub(super) fn enumeration(
    data: DataEnum,
    name: &str,
    span: proc_macro2::Span,
    dataclass: crate::attrs::DataclassOptions,
    params: &[String],
) -> syn::Result<ModelPieces> {
    enumeration::render(data, name, span, dataclass, params)
}
