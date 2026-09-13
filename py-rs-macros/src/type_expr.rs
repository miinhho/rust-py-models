use proc_macro2::TokenStream as Tokens;
use quote::quote;
use std::collections::HashSet;
use syn::{GenericArgument, PathArguments, Type};

fn type_args(ty: &Type) -> Vec<&Type> {
    match ty {
        Type::Path(path) => path
            .path
            .segments
            .iter()
            .flat_map(|segment| match &segment.arguments {
                PathArguments::AngleBracketed(args) => args
                    .args
                    .iter()
                    .filter_map(|arg| {
                        if let GenericArgument::Type(ty) = arg {
                            Some(ty)
                        } else {
                            None
                        }
                    })
                    .collect::<Vec<_>>(),
                _ => Vec::new(),
            })
            .collect(),
        Type::Reference(reference) => vec![&reference.elem],
        Type::Slice(slice) => vec![&slice.elem],
        Type::Array(array) => vec![&array.elem],
        Type::Tuple(tuple) => tuple.elems.iter().collect(),
        Type::Paren(paren) => vec![&paren.elem],
        Type::Group(group) => vec![&group.elem],
        _ => Vec::new(),
    }
}

fn is_param(ty: &Type, params: &HashSet<String>) -> Option<String> {
    if let Type::Path(path) = ty {
        if path.qself.is_none() && path.path.segments.len() == 1 {
            let segment = path.path.segments.first()?;
            if segment.arguments.is_empty() && params.contains(&segment.ident.to_string()) {
                return Some(segment.ident.to_string());
            }
        }
    }
    None
}

pub(crate) fn inline(ty: &Type, params: &HashSet<String>) -> Tokens {
    if let Some(param) = is_param(ty, params) {
        return quote! { String::from(#param) };
    }
    match ty {
        Type::Paren(paren) => return inline(&paren.elem, params),
        Type::Group(group) => return inline(&group.elem, params),
        Type::Reference(reference) => return inline(&reference.elem, params),
        _ => {}
    }
    let args = type_args(ty)
        .into_iter()
        .map(|arg| inline(arg, params))
        .collect::<Vec<_>>();
    quote! { <#ty as ::py_rs::PY>::inline_with(&[#(#args),*]) }
}

pub(crate) fn dependencies(ty: &Type, params: &HashSet<String>) -> Vec<Tokens> {
    if is_param(ty, params).is_some() {
        return Vec::new();
    }
    let mut out = match ty {
        Type::Path(_) => vec![
            quote! { if let Some(dep) = <#ty as ::py_rs::PY>::dependency() { deps.push(dep); } },
        ],
        _ => Vec::new(),
    };
    for arg in type_args(ty) {
        out.extend(dependencies(arg, params));
    }
    out
}

pub(crate) fn imports(ty: &Type, params: &HashSet<String>) -> Vec<Tokens> {
    if is_param(ty, params).is_some() {
        return Vec::new();
    }
    let mut out = match ty {
        Type::Path(_) | Type::Array(_) | Type::Slice(_) | Type::Tuple(_) => {
            vec![quote! { imports.extend(<#ty as ::py_rs::PY>::local_imports()); }]
        }
        _ => Vec::new(),
    };
    for arg in type_args(ty) {
        out.extend(imports(arg, params));
    }
    out
}

pub(crate) fn definitions(ty: &Type, params: &HashSet<String>) -> Vec<Tokens> {
    if is_param(ty, params).is_some() {
        return Vec::new();
    }
    let mut out = match ty {
        Type::Path(_) | Type::Array(_) | Type::Slice(_) | Type::Tuple(_) => {
            vec![quote! { definitions.extend(<#ty as ::py_rs::PY>::annotation_definitions()); }]
        }
        _ => Vec::new(),
    };
    for arg in type_args(ty) {
        out.extend(definitions(arg, params));
    }
    out
}
