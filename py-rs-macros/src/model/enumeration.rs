use super::ModelPieces;
use crate::attrs::{options, DataclassOptions, Options};
use crate::fields::{fields, FieldPieces};
use crate::python::{python_class_ident, python_ident, python_literal};
use proc_macro2::Span;
use quote::quote;
use std::collections::HashSet;
use syn::{DataEnum, Fields, Variant};

pub(super) fn render(
    data: DataEnum,
    name: &str,
    span: Span,
    dataclass: DataclassOptions,
    params: &[String],
) -> syn::Result<ModelPieces> {
    let mut variants = Vec::new();
    for variant in data.variants {
        let attr = options(&variant.attrs)?;
        if attr.export
            || attr.export_to.is_some()
            || attr.python_type.is_some()
            || attr.import.is_some()
        {
            return Err(syn::Error::new_spanned(
                variant,
                "export and type override options do not apply to enum variants",
            ));
        }
        if !attr.skip {
            variants.push((variant, attr));
        }
    }
    if variants
        .iter()
        .all(|(variant, _)| matches!(variant.fields, Fields::Unit))
    {
        render_unit(variants, name, span, dataclass)
    } else {
        render_payload(variants, name, dataclass, params)
    }
}

fn render_unit(
    variants: Vec<(Variant, Options)>,
    name: &str,
    span: Span,
    dataclass: DataclassOptions,
) -> syn::Result<ModelPieces> {
    if dataclass.is_set() {
        return Err(syn::Error::new(
            span,
            "dataclass options do not apply to a unit enum",
        ));
    }
    let mut names = HashSet::new();
    let mut members = Vec::new();
    for (variant, attr) in variants {
        if attr.dataclass.is_set() {
            return Err(syn::Error::new_spanned(
                variant,
                "dataclass options do not apply to a unit enum",
            ));
        }
        let ident = rust_variant_name(&variant)?;
        let value = attr.rename.unwrap_or_else(|| ident.clone());
        if !names.insert(value.clone()) {
            return Err(syn::Error::new_spanned(
                variant,
                format!("duplicate Python enum value {value:?}"),
            ));
        }
        let literal = python_literal(&value);
        members.push(quote! {
            out.push_str(&format!("    {} = {}\n", #ident, #literal));
        });
    }
    let pass = if members.is_empty() {
        quote! { out.push_str("    pass\n"); }
    } else {
        quote! {}
    };
    Ok(ModelPieces {
        prelude: quote! { String::from("from enum import Enum\n") },
        declaration: quote! {
            let mut out = format!("class {}(str, Enum):\n", #name);
            #(#members)*
            #pass
            out
        },
        deps: Vec::new(),
        imports: Vec::new(),
        definitions: Vec::new(),
        validations: Vec::new(),
        hashable: quote! { true },
        local_names: vec![name.to_owned()],
    })
}

fn render_payload(
    variants: Vec<(Variant, Options)>,
    name: &str,
    dataclass: DataclassOptions,
    params: &[String],
) -> syn::Result<ModelPieces> {
    let mut deps = Vec::new();
    let mut imports = Vec::new();
    let mut definitions = Vec::new();
    let mut validations = Vec::new();
    let mut hash_checks = Vec::new();
    let mut local_names = vec![name.to_owned()];
    let mut names = HashSet::new();
    let mut classes = Vec::new();
    let param_set = params.iter().cloned().collect();
    let generic = if params.is_empty() {
        String::new()
    } else {
        format!("(Generic[{}])", params.join(", "))
    };

    for (variant, attr) in variants {
        let ident = rust_variant_name(&variant)?;
        let suffix = attr.rename.unwrap_or(ident);
        python_ident(&suffix, variant.ident.span())?;
        let class_name = format!("{name}{suffix}");
        python_class_ident(&class_name, variant.ident.span())?;
        if !names.insert(class_name.clone()) {
            return Err(syn::Error::new_spanned(
                variant,
                format!("duplicate Python variant class {class_name:?}"),
            ));
        }
        let variant_options = dataclass.with_overrides(attr.dataclass);
        let decorator = variant_options.decorator();
        let FieldPieces {
            lines,
            deps: field_deps,
            imports: field_imports,
            definitions: field_definitions,
            hash_checks: field_hashes,
            validations: field_validations,
            empty,
        } = fields(&variant.fields, &param_set)?;
        deps.extend(field_deps);
        imports.extend(field_imports);
        definitions.extend(field_definitions);
        validations.extend(field_validations);
        let frozen = variant_options.frozen.unwrap_or(false);
        hash_checks.push(quote! { #frozen #(&& #field_hashes)* });
        local_names.push(class_name.clone());
        let pass = if empty {
            quote! { out.push_str("    pass\n"); }
        } else {
            quote! {}
        };
        classes.push(quote! {
            out.push_str(&format!("{}\nclass {}{}:\n", #decorator, #class_name, #generic));
            #(#lines)*
            #pass
            out.push('\n');
        });
    }
    let type_args = if params.is_empty() {
        String::new()
    } else {
        format!("[{}]", params.join(", "))
    };
    let alias = local_names[1..]
        .iter()
        .map(|name| format!("{name}{type_args}"))
        .collect::<Vec<_>>()
        .join(" | ");
    let prelude = if params.is_empty() {
        quote! { String::from("from dataclasses import dataclass\nfrom typing import TypeAlias\n") }
    } else {
        quote! { String::from("from dataclasses import dataclass\nfrom typing import Generic, TypeAlias, TypeVar\n") }
    };
    Ok(ModelPieces {
        prelude,
        declaration: quote! {
            let mut out = String::new();
            #(#classes)*
            out.push_str(&format!("{}: TypeAlias = {}\n", #name, #alias));
            out
        },
        deps,
        imports,
        definitions,
        validations,
        hashable: quote! { true #(&& #hash_checks)* },
        local_names,
    })
}

fn rust_variant_name(variant: &Variant) -> syn::Result<String> {
    let ident = variant
        .ident
        .to_string()
        .trim_start_matches("r#")
        .to_owned();
    python_ident(&ident, variant.ident.span())?;
    Ok(ident)
}
