use crate::attrs::{options, Options};
use crate::fields::is_phantom_data;
use crate::python::python_class_ident;
use crate::type_expr::{result_args, type_args};
use proc_macro2::TokenStream as Tokens;
use quote::quote;
use std::collections::{HashMap, HashSet};
use syn::fold::{self, Fold};
use syn::{Data, DataEnum, DataStruct, Field, Fields, GenericParam, Generics, Ident, Type};

pub(crate) struct GenericInfo {
    pub(crate) impl_generics: Generics,
    pub(crate) names: Vec<String>,
    pub(crate) all_names: Vec<String>,
    type_params: Vec<Ident>,
    visible_indices: Vec<usize>,
    concrete: HashMap<String, Type>,
}

impl GenericInfo {
    pub(crate) fn parse(original: &Generics, options: &Options, data: &Data) -> syn::Result<Self> {
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

        // Error-only parameters have no Python representation and need no PY bound.
        let hidden = hidden_result_error_params(data, options, &declared)?;
        let mut impl_generics = original.clone();
        let all_names = original
            .type_params()
            .map(|param| param.ident.to_string())
            .collect();
        let visible_indices = original
            .type_params()
            .enumerate()
            .filter(|(_, param)| {
                !concrete.contains_key(&param.ident.to_string())
                    && !hidden.contains(&param.ident.to_string())
            })
            .map(|(index, _)| index)
            .collect::<Vec<_>>();
        let type_params = original
            .type_params()
            .filter(|param| {
                !concrete.contains_key(&param.ident.to_string())
                    && !hidden.contains(&param.ident.to_string())
            })
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
            all_names,
            type_params,
            visible_indices,
            concrete,
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

    pub(crate) fn concrete_specs(&self) -> Vec<Tokens> {
        self.type_params
            .iter()
            .map(|param| quote! { <#param as ::rust_py_models::PY>::type_spec() })
            .collect()
    }

    pub(crate) fn supplied_specs(&self) -> Vec<Tokens> {
        self.visible_indices
            .iter()
            .map(|index| quote! { args[#index].clone() })
            .collect()
    }
}

fn hidden_result_error_params(
    data: &Data,
    container: &Options,
    declared: &HashSet<String>,
) -> syn::Result<HashSet<String>> {
    let mut visible = HashSet::new();
    let mut error_only = HashSet::new();
    if let Some(ty) = &container.as_type {
        collect_visible(ty, declared, &mut visible, &mut error_only);
    } else {
        let mut visit_field = |field: &Field| -> syn::Result<()> {
            let attr = options(&field.attrs)?;
            if attr.is_skipped()
                || attr.unsafe_python_type.is_some()
                || (attr.as_type.is_none() && is_phantom_data(&field.ty))
            {
                return Ok(());
            }
            collect_visible(
                attr.as_type.as_ref().unwrap_or(&field.ty),
                declared,
                &mut visible,
                &mut error_only,
            );
            Ok(())
        };
        let mut visit_fields = |fields: &Fields| -> syn::Result<()> {
            for field in fields {
                visit_field(field)?;
            }
            Ok(())
        };
        match data {
            Data::Struct(data) => visit_fields(&data.fields)?,
            Data::Enum(data) => {
                for variant in &data.variants {
                    if !options(&variant.attrs)?.is_skipped() {
                        visit_fields(&variant.fields)?;
                    }
                }
            }
            Data::Union(_) => {}
        }
    }
    Ok(error_only.difference(&visible).cloned().collect())
}

fn collect_visible(
    ty: &Type,
    declared: &HashSet<String>,
    visible: &mut HashSet<String>,
    error_only: &mut HashSet<String>,
) {
    if let Some((ok, err)) = result_args(ty) {
        collect_visible(ok, declared, visible, error_only);
        collect_all_params(err, declared, error_only);
    } else {
        if let Type::Path(path) = ty {
            if path.qself.is_none() && path.path.segments.len() == 1 {
                let segment = &path.path.segments[0];
                if segment.arguments.is_empty() && declared.contains(&segment.ident.to_string()) {
                    visible.insert(segment.ident.to_string());
                }
            }
        }
        for arg in type_args(ty) {
            collect_visible(arg, declared, visible, error_only);
        }
    }
}

fn collect_all_params(ty: &Type, declared: &HashSet<String>, out: &mut HashSet<String>) {
    if let Type::Path(path) = ty {
        if path.qself.is_none() && path.path.segments.len() == 1 {
            let segment = &path.path.segments[0];
            if segment.arguments.is_empty() && declared.contains(&segment.ident.to_string()) {
                out.insert(segment.ident.to_string());
            }
        }
    }
    for arg in type_args(ty) {
        collect_all_params(arg, declared, out);
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
