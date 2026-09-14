use crate::attrs::options;
use crate::derive;
use crate::generics::visible_params;
use crate::projection::ProjectionMap;
use proc_macro2::TokenStream as Tokens;
use quote::quote;
use std::collections::{BTreeMap, HashSet};
use syn::parse::Parser;
use syn::punctuated::Punctuated;
use syn::{Attribute, Data, DeriveInput, GenericParam, Item, ItemMod, Path, Token};

pub(crate) fn expand(mut module: ItemMod) -> syn::Result<Tokens> {
    let Some((_, items)) = &module.content else {
        return Err(syn::Error::new_spanned(
            &module,
            "#[py_models] requires an inline module",
        ));
    };
    let mut inputs = BTreeMap::<String, DeriveInput>::new();
    for item in items {
        let attrs = match item {
            Item::Struct(item) => &item.attrs,
            Item::Enum(item) => &item.attrs,
            _ => continue,
        };
        if !has_py_derive(attrs)? {
            continue;
        }
        let input = syn::parse2::<DeriveInput>(quote! { #item })?;
        let name = input.ident.to_string();
        if inputs.insert(name.clone(), input).is_some() {
            return Err(syn::Error::new_spanned(
                item,
                format!("duplicate grouped model {name:?}"),
            ));
        }
    }

    let mut projection = ProjectionMap::default();
    for (name, input) in &inputs {
        projection.models.insert(
            name.clone(),
            vec![false; input.generics.type_params().count()],
        );
    }
    loop {
        let mut changed = false;
        for (name, input) in &inputs {
            let declared = input
                .generics
                .type_params()
                .map(|param| param.ident.to_string())
                .collect::<HashSet<_>>();
            let attr = options(&input.attrs)?;
            let data = match &input.data {
                Data::Struct(_) | Data::Enum(_) => &input.data,
                Data::Union(_) => continue,
            };
            let visible = visible_params(data, &attr, &declared, &projection)?;
            let concrete = attr
                .concrete
                .iter()
                .map(|(ident, _)| ident.to_string())
                .collect::<HashSet<_>>();
            let slots = input
                .generics
                .params
                .iter()
                .filter_map(|param| match param {
                    GenericParam::Type(param) => Some(param.ident.to_string()),
                    _ => None,
                })
                .map(|param| visible.contains(&param) && !concrete.contains(&param))
                .collect::<Vec<_>>();
            let current = projection
                .models
                .get_mut(name)
                .expect("model is registered");
            if *current != slots {
                *current = slots;
                changed = true;
            }
        }
        if !changed {
            break;
        }
    }

    let mut implementations = Vec::new();
    for input in inputs.values() {
        let slots = &projection.models[&input.ident.to_string()];
        let names = input
            .generics
            .type_params()
            .zip(slots)
            .filter(|(_, used)| **used)
            .map(|(param, _)| param.ident.to_string())
            .collect::<HashSet<_>>();
        let cfg = input
            .attrs
            .iter()
            .filter(|attr| attr.path().is_ident("cfg"))
            .cloned()
            .collect::<Vec<_>>();
        implementations.push((
            cfg,
            derive::expand_with(input.clone(), &projection, Some(&names))?,
        ));
    }

    let (_, items) = module.content.as_mut().expect("inline module");
    for item in items {
        match item {
            Item::Struct(item) if inputs.contains_key(&item.ident.to_string()) => {
                strip_attrs(&mut item.attrs)?;
                for field in &mut item.fields {
                    field.attrs.retain(|attr| !attr.path().is_ident("py"));
                }
            }
            Item::Enum(item) if inputs.contains_key(&item.ident.to_string()) => {
                strip_attrs(&mut item.attrs)?;
                for variant in &mut item.variants {
                    variant.attrs.retain(|attr| !attr.path().is_ident("py"));
                    for field in &mut variant.fields {
                        field.attrs.retain(|attr| !attr.path().is_ident("py"));
                    }
                }
            }
            _ => {}
        }
    }
    let (brace, mut items) = module.content.take().expect("inline module");
    for (cfg, implementation) in implementations {
        for mut item in syn::parse2::<syn::File>(implementation)?.items {
            match &mut item {
                Item::Impl(item) => item.attrs.extend(cfg.clone()),
                Item::Macro(item) => item.attrs.extend(cfg.clone()),
                _ => {}
            }
            items.push(item);
        }
    }
    module.content = Some((brace, items));
    Ok(quote! { #module })
}

fn has_py_derive(attrs: &[Attribute]) -> syn::Result<bool> {
    for attr in attrs.iter().filter(|attr| attr.path().is_ident("derive")) {
        let derives = Punctuated::<Path, Token![,]>::parse_terminated
            .parse2(attr.meta.require_list()?.tokens.clone())?;
        if derives.iter().any(|path| {
            path.segments
                .last()
                .is_some_and(|segment| segment.ident == "PY")
        }) {
            return Ok(true);
        }
    }
    Ok(false)
}

fn strip_attrs(attrs: &mut Vec<Attribute>) -> syn::Result<()> {
    let mut clean = Vec::new();
    for attr in attrs.drain(..) {
        if attr.path().is_ident("py") {
            continue;
        }
        if !attr.path().is_ident("derive") {
            clean.push(attr);
            continue;
        }
        let derives = Punctuated::<Path, Token![,]>::parse_terminated
            .parse2(attr.meta.require_list()?.tokens.clone())?;
        let keep = derives
            .into_iter()
            .filter(|path| {
                path.segments
                    .last()
                    .is_none_or(|segment| segment.ident != "PY")
            })
            .collect::<Vec<_>>();
        if !keep.is_empty() {
            clean.push(syn::parse_quote!(#[derive(#(#keep),*)]));
        }
    }
    *attrs = clean;
    Ok(())
}
