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

pub(crate) fn result_args(ty: &Type) -> Option<(&Type, &Type)> {
    let Type::Path(path) = ty else {
        return None;
    };
    if path.qself.is_some() {
        return None;
    }
    let segments = path.path.segments.iter().collect::<Vec<_>>();
    let is_result = match segments.as_slice() {
        [result] => result.ident == "Result",
        [root, module, result] => {
            (root.ident == "std" || root.ident == "core")
                && module.ident == "result"
                && result.ident == "Result"
        }
        _ => false,
    };
    if !is_result {
        return None;
    }
    let PathArguments::AngleBracketed(arguments) = &segments.last()?.arguments else {
        return None;
    };
    let mut args = arguments.args.iter();
    let (Some(GenericArgument::Type(ok)), Some(GenericArgument::Type(err)), None) =
        (args.next(), args.next(), args.next())
    else {
        return None;
    };
    Some((ok, err))
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

fn contains_param(ty: &Type, params: &HashSet<String>) -> bool {
    is_param(ty, params).is_some()
        || type_args(ty)
            .into_iter()
            .any(|arg| contains_param(arg, params))
}

pub(crate) fn symbolic(ty: &Type, params: &HashSet<String>) -> Tokens {
    if let Some((ok, _)) = result_args(ty) {
        return symbolic(ok, params);
    }
    if let Some(param) = is_param(ty, params) {
        return quote! {
            ::rust_py_models::TypeSpec::symbolic(
                #param,
                &<#ty as ::rust_py_models::PY>::type_spec(),
            )
        };
    }
    if !contains_param(ty, params) {
        return quote! { <#ty as ::rust_py_models::PY>::type_spec() };
    }
    match ty {
        Type::Paren(paren) => return symbolic(&paren.elem, params),
        Type::Group(group) => return symbolic(&group.elem, params),
        Type::Reference(reference) => return symbolic(&reference.elem, params),
        _ => {}
    }
    let args = type_args(ty)
        .into_iter()
        .map(|arg| symbolic(arg, params))
        .collect::<Vec<_>>();
    quote! { <#ty as ::rust_py_models::PY>::type_spec_with(&[#(#args),*]) }
}

pub(crate) fn concrete(ty: &Type, params: &HashSet<String>) -> Tokens {
    if let Some((ok, _)) = result_args(ty) {
        return concrete(ok, params);
    }
    if is_param(ty, params).is_some() {
        return quote! { <#ty as ::rust_py_models::PY>::type_spec() };
    }
    match ty {
        Type::Paren(paren) => return concrete(&paren.elem, params),
        Type::Group(group) => return concrete(&group.elem, params),
        Type::Reference(reference) => return concrete(&reference.elem, params),
        _ => {}
    }
    let args = type_args(ty)
        .into_iter()
        .map(|arg| concrete(arg, params))
        .collect::<Vec<_>>();
    quote! { <#ty as ::rust_py_models::PY>::type_spec_with(&[#(#args),*]) }
}

pub(crate) fn supplied(ty: &Type, params: &[String]) -> Tokens {
    let params_set = params.iter().cloned().collect::<HashSet<_>>();
    supplied_inner(ty, params, &params_set)
}

fn supplied_inner(ty: &Type, params: &[String], params_set: &HashSet<String>) -> Tokens {
    if let Some((ok, _)) = result_args(ty) {
        return supplied_inner(ok, params, params_set);
    }
    if let Some(param) = is_param(ty, params_set) {
        let index = params
            .iter()
            .position(|candidate| candidate == &param)
            .expect("generic parameter is present");
        return quote! { args[#index].clone() };
    }
    match ty {
        Type::Paren(paren) => return supplied_inner(&paren.elem, params, params_set),
        Type::Group(group) => return supplied_inner(&group.elem, params, params_set),
        Type::Reference(reference) => return supplied_inner(&reference.elem, params, params_set),
        _ => {}
    }
    let children = type_args(ty)
        .into_iter()
        .map(|arg| supplied_inner(arg, params, params_set))
        .collect::<Vec<_>>();
    quote! { <#ty as ::rust_py_models::PY>::type_spec_with(&[#(#children),*]) }
}
