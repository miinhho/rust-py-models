use super::{dataclass_options, documentation, ModelPieces};
use crate::attrs::Options;
use crate::fields::{fields, FieldPieces};
use crate::python::python_ident;
use quote::quote;
use std::collections::HashSet;
use syn::DataStruct;

pub(super) fn render(
    data: DataStruct,
    name: &str,
    attr: &Options,
    params: &[String],
    projection: &crate::projection::ProjectionMap,
) -> syn::Result<ModelPieces> {
    if attr.serde_rename_all_fields.is_some()
        || attr.serde_content.is_some()
        || attr.serde_untagged
        || attr.serde_flatten
    {
        return Err(syn::Error::new_spanned(
            data.struct_token,
            "enum-only or field-only serde attributes cannot be used on a struct",
        ));
    }
    if attr.newtype {
        return render_newtype(data, name, attr, params, projection);
    }
    let documentation = documentation(&attr.documentation);
    let dataclass = attr.dataclass;
    let param_set = params.iter().cloned().collect();
    let FieldPieces {
        mut statements,
        hash_checks,
        ..
    } = fields(&data.fields, &param_set, attr.serde_rename_all, projection)?;
    if let Some(tag) = &attr.serde_tag {
        python_ident(tag, proc_macro2::Span::call_site())?;
        statements.insert(
            0,
            quote! {
                fields.push(
                    ::rust_py_models::FieldSpec::new(
                        #tag,
                        ::rust_py_models::TypeSpec::literal(#name),
                    )
                    .with_default(::rust_py_models::FieldDefault::DataclassField {
                        init: false,
                        value: String::from(#name),
                    }),
                );
            },
        );
    }
    let frozen = dataclass.frozen.unwrap_or(false);
    let hashable = quote! { #frozen #(&& #hash_checks)* };
    let options = dataclass_options(dataclass);
    Ok(ModelPieces {
        declarations: vec![quote! {
            {
                let mut fields = Vec::new();
                #(#statements)*
                ::rust_py_models::Declaration::Dataclass(
                    ::rust_py_models::DataclassSpec {
                        name: String::from(#name),
                        documentation: #documentation,
                        options: #options,
                        type_params: vec![#(String::from(#params)),*],
                        fields,
                    },
                )
            }
        }],
        hashable,
    })
}

fn render_newtype(
    data: DataStruct,
    name: &str,
    attr: &Options,
    params: &[String],
    projection: &crate::projection::ProjectionMap,
) -> syn::Result<ModelPieces> {
    if attr.dataclass.is_set() {
        return Err(syn::Error::new_spanned(
            data.struct_token,
            "dataclass options do not apply to #[py(newtype)]",
        ));
    }
    if !params.is_empty() {
        return Err(syn::Error::new_spanned(
            data.struct_token,
            "generic #[py(newtype)] declarations are not supported by typing.NewType",
        ));
    }
    if attr.serde_tag.is_some() || attr.serde_rename_all.is_some() {
        return Err(syn::Error::new_spanned(
            data.struct_token,
            "Serde shape options do not apply to #[py(newtype)]",
        ));
    }
    let syn::Fields::Unnamed(unnamed) = &data.fields else {
        return Err(syn::Error::new_spanned(
            data.fields,
            "#[py(newtype)] requires a one-field tuple struct",
        ));
    };
    if unnamed.unnamed.len() != 1 {
        return Err(syn::Error::new_spanned(
            unnamed,
            "#[py(newtype)] requires a one-field tuple struct",
        ));
    }
    let FieldPieces {
        specs, hash_checks, ..
    } = fields(&data.fields, &HashSet::new(), None, projection)?;
    let Some(target) = specs.into_iter().next() else {
        return Err(syn::Error::new_spanned(
            unnamed,
            "#[py(newtype)] field cannot be skipped",
        ));
    };
    let hashable = hash_checks
        .into_iter()
        .next()
        .expect("one-field newtype has one hashability check");
    let documentation = documentation(&attr.documentation);
    Ok(ModelPieces {
        declarations: vec![quote! {
            ::rust_py_models::Declaration::NewType {
                name: String::from(#name),
                target: #target,
                documentation: #documentation,
            }
        }],
        hashable,
    })
}
