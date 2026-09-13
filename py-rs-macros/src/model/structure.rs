use super::ModelPieces;
use crate::attrs::DataclassOptions;
use crate::fields::{fields, FieldPieces};
use quote::quote;
use syn::DataStruct;

pub(super) fn render(
    data: DataStruct,
    name: &str,
    dataclass: DataclassOptions,
    params: &[String],
) -> syn::Result<ModelPieces> {
    let param_set = params.iter().cloned().collect();
    let FieldPieces {
        lines,
        deps,
        imports,
        definitions,
        hash_checks,
        validations,
        empty,
    } = fields(&data.fields, &param_set)?;
    let frozen = dataclass.frozen.unwrap_or(false);
    let hashable = quote! { #frozen #(&& #hash_checks)* };
    let decorator = dataclass.decorator();
    let pass = if empty {
        quote! { out.push_str("    pass\n"); }
    } else {
        quote! {}
    };
    let generic = if params.is_empty() {
        String::new()
    } else {
        format!("(Generic[{}])", params.join(", "))
    };
    let prelude = if params.is_empty() {
        quote! { String::from("from dataclasses import dataclass\n") }
    } else {
        quote! { String::from("from dataclasses import dataclass\nfrom typing import Generic, TypeVar\n") }
    };
    Ok(ModelPieces {
        prelude,
        declaration: quote! {
            let mut out = format!("{}\nclass {}{}:\n", #decorator, #name, #generic);
            #(#lines)*
            #pass
            out
        },
        deps,
        imports,
        definitions,
        validations,
        hashable,
        local_names: vec![name.to_owned()],
    })
}
