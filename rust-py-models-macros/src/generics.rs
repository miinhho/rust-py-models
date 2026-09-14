use crate::attrs::Options;
use crate::python::python_class_ident;
use proc_macro2::TokenStream as Tokens;
use quote::quote;
use std::collections::{HashMap, HashSet};
use syn::fold::{self, Fold};
use syn::{DataEnum, DataStruct, GenericParam, Generics, Ident, Type};

pub(crate) struct GenericInfo {
    pub(crate) impl_generics: Generics,
    pub(crate) names: Vec<String>,
    type_params: Vec<Ident>,
    concrete: HashMap<String, Type>,
    has_params: bool,
}

impl GenericInfo {
    pub(crate) fn parse(original: &Generics, options: &Options) -> syn::Result<Self> {
        if let Some(param) = original
            .params
            .iter()
            .find(|param| matches!(param, GenericParam::Const(_)))
        {
            return Err(syn::Error::new_spanned(
                param,
                "const generics are not supported by PY derive",
            ));
        }
        let declared = original
            .type_params()
            .map(|param| param.ident.to_string())
            .collect::<HashSet<_>>();
        let mut concrete = HashMap::new();
        for (ident, ty) in &options.concrete {
            let name = ident.to_string();
            if !declared.contains(&name) {
                return Err(syn::Error::new_spanned(
                    ident,
                    format!("unknown concrete generic parameter {name:?}"),
                ));
            }
            if concrete.insert(name.clone(), ty.clone()).is_some() {
                return Err(syn::Error::new_spanned(
                    ident,
                    format!("duplicate concrete generic parameter {name:?}"),
                ));
            }
        }

        let mut impl_generics = original.clone();
        let type_params = original
            .type_params()
            .filter(|param| !concrete.contains_key(&param.ident.to_string()))
            .map(|param| param.ident.clone())
            .collect::<Vec<_>>();
        for param in &type_params {
            python_class_ident(&param.to_string(), param.span())?;
        }
        if let Some(predicates) = &options.bound {
            impl_generics
                .make_where_clause()
                .predicates
                .extend(predicates.iter().cloned());
        } else {
            for param in &type_params {
                impl_generics
                    .make_where_clause()
                    .predicates
                    .push(syn::parse_quote!(#param: ::rust_py_models::PY));
            }
        }
        let names = type_params.iter().map(ToString::to_string).collect();
        Ok(Self {
            impl_generics,
            names,
            type_params,
            concrete,
            has_params: !original.params.is_empty(),
        })
    }

    pub(crate) fn concretize_struct(&self, data: DataStruct) -> DataStruct {
        ConcreteFolder {
            concrete: &self.concrete,
        }
        .fold_data_struct(data)
    }

    pub(crate) fn concretize_enum(&self, data: DataEnum) -> DataEnum {
        ConcreteFolder {
            concrete: &self.concrete,
        }
        .fold_data_enum(data)
    }
    pub(crate) fn concretize_type(&self, ty: Type) -> Type {
        ConcreteFolder {
            concrete: &self.concrete,
        }
        .fold_type(ty)
    }

    pub(crate) fn has_params(&self) -> bool {
        self.has_params
    }

    pub(crate) fn concrete_specs(&self) -> Vec<Tokens> {
        self.type_params
            .iter()
            .map(|param| quote! { <#param as ::rust_py_models::PY>::type_spec() })
            .collect()
    }
}

struct ConcreteFolder<'a> {
    concrete: &'a HashMap<String, Type>,
}

impl Fold for ConcreteFolder<'_> {
    fn fold_type(&mut self, ty: Type) -> Type {
        if let Type::Path(path) = &ty {
            if path.qself.is_none() && path.path.segments.len() == 1 {
                let segment = &path.path.segments[0];
                if segment.arguments.is_empty() {
                    if let Some(replacement) = self.concrete.get(&segment.ident.to_string()) {
                        return replacement.clone();
                    }
                }
            }
        }
        fold::fold_type(self, ty)
    }
}
