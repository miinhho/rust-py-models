use crate::projection::ProjectionMap;
use proc_macro2::TokenStream as Tokens;
use quote::quote;
use std::collections::HashSet;
use syn::{GenericArgument, PathArguments, Type};

pub(crate) fn type_args(ty: &Type) -> Vec<&Type> {
    match ty {
        Type::Path(path) => path
            .path
            .segments
            .iter()
            .flat_map(|segment| match &segment.arguments {
                PathArguments::AngleBracketed(args) => args
                    .args
                    .iter()
                    .filter_map(|arg| match arg {
                        GenericArgument::Type(ty) => Some(ty),
                        _ => None,
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

pub(crate) fn is_param(ty: &Type, params: &HashSet<String>) -> Option<String> {
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

fn contains_param(ty: &Type, params: &HashSet<String>, projection: &ProjectionMap) -> bool {
    is_param(ty, params).is_some()
        || projection
            .args(ty)
            .into_iter()
            .any(|(used, arg)| used && contains_param(arg, params, projection))
}

pub(crate) fn symbolic(ty: &Type, params: &HashSet<String>, projection: &ProjectionMap) -> Tokens {
    if let Some(param) = is_param(ty, params) {
        return quote! {
            ::rust_py_models::TypeSpec::symbolic(
                #param,
                &<#ty as ::rust_py_models::PY>::type_spec(),
            )
        };
    }
    if !contains_param(ty, params, projection) {
        return quote! { <#ty as ::rust_py_models::PY>::type_spec() };
    }
    match ty {
        Type::Paren(paren) => return symbolic(&paren.elem, params, projection),
        Type::Group(group) => return symbolic(&group.elem, params, projection),
        Type::Reference(reference) => return symbolic(&reference.elem, params, projection),
        _ => {}
    }
    let args = projection
        .args(ty)
        .into_iter()
        .map(|(used, arg)| {
            if used {
                symbolic(arg, params, projection)
            } else {
                quote! { ::rust_py_models::TypeSpec::named("__ignored") }
            }
        })
        .collect::<Vec<_>>();
    quote! { <#ty as ::rust_py_models::PY>::type_spec_with(&[#(#args),*]) }
}

pub(crate) fn concrete(ty: &Type, params: &HashSet<String>, projection: &ProjectionMap) -> Tokens {
    if is_param(ty, params).is_some() {
        return quote! { <#ty as ::rust_py_models::PY>::type_spec() };
    }
    match ty {
        Type::Paren(paren) => return concrete(&paren.elem, params, projection),
        Type::Group(group) => return concrete(&group.elem, params, projection),
        Type::Reference(reference) => return concrete(&reference.elem, params, projection),
        _ => {}
    }
    let args = projection
        .args(ty)
        .into_iter()
        .map(|(used, arg)| {
            if used {
                concrete(arg, params, projection)
            } else {
                quote! { ::rust_py_models::TypeSpec::named("__ignored") }
            }
        })
        .collect::<Vec<_>>();
    quote! { <#ty as ::rust_py_models::PY>::type_spec_with(&[#(#args),*]) }
}

pub(crate) fn supplied(ty: &Type, params: &[String], projection: &ProjectionMap) -> Tokens {
    let params_set = params.iter().cloned().collect::<HashSet<_>>();
    supplied_inner(ty, params, &params_set, projection)
}

fn supplied_inner(
    ty: &Type,
    params: &[String],
    params_set: &HashSet<String>,
    projection: &ProjectionMap,
) -> Tokens {
    if let Some(param) = is_param(ty, params_set) {
        let index = params
            .iter()
            .position(|candidate| candidate == &param)
            .expect("generic parameter is present");
        return quote! { args[#index].clone() };
    }
    match ty {
        Type::Paren(paren) => return supplied_inner(&paren.elem, params, params_set, projection),
        Type::Group(group) => return supplied_inner(&group.elem, params, params_set, projection),
        Type::Reference(reference) => {
            return supplied_inner(&reference.elem, params, params_set, projection)
        }
        _ => {}
    }
    let children = projection
        .args(ty)
        .into_iter()
        .map(|(used, arg)| {
            if used {
                supplied_inner(arg, params, params_set, projection)
            } else {
                quote! { ::rust_py_models::TypeSpec::named("__ignored") }
            }
        })
        .collect::<Vec<_>>();
    quote! { <#ty as ::rust_py_models::PY>::type_spec_with(&[#(#children),*]) }
}
