use crate::attrs::{options, Options};
use crate::fields::{project_field, FieldProjection};
use crate::python::python_class_ident;
use crate::type_expr::{is_param, result_args, type_args};
use proc_macro2::TokenStream as Tokens;
use quote::quote;
use std::collections::{HashMap, HashSet};
use syn::fold::{self, Fold};
use syn::{Data, DataEnum, DataStruct, Field, Fields, GenericParam, Generics, Ident, Type};

pub(crate) struct GenericInfo {
    pub(crate) impl_generics: Generics,
    pub(crate) names: Vec<String>,
    pub(crate) all_names: Vec<String>,
    exposed_params: Vec<ExposedParam>,
    concrete: HashMap<String, Type>,
}

struct ExposedParam {
    ident: Ident,
    rust_index: usize,
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
        let all_names = original
            .type_params()
            .map(|param| param.ident.to_string())
            .collect::<Vec<_>>();
        let declared = all_names.iter().cloned().collect::<HashSet<_>>();
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
        let exposed_params = original
            .type_params()
            .enumerate()
            .filter(|(index, _)| {
                !concrete.contains_key(&all_names[*index]) && !hidden.contains(&all_names[*index])
            })
            .map(|(rust_index, param)| ExposedParam {
                ident: param.ident.clone(),
                rust_index,
            })
            .collect::<Vec<_>>();
        for param in &exposed_params {
            python_class_ident(&param.ident.to_string(), param.ident.span())?;
        }
        if let Some(predicates) = &options.bound {
            impl_generics
                .make_where_clause()
                .predicates
                .extend(predicates.iter().cloned());
        } else {
            for param in &exposed_params {
                let ident = &param.ident;
                impl_generics
                    .make_where_clause()
                    .predicates
                    .push(syn::parse_quote!(#ident: ::rust_py_models::PY));
            }
        }
        let names = exposed_params
            .iter()
            .map(|param| param.ident.to_string())
            .collect();
        Ok(Self {
            impl_generics,
            names,
            all_names,
            exposed_params,
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
        self.exposed_params
            .iter()
            .map(|param| {
                let ident = &param.ident;
                quote! { <#ident as ::rust_py_models::PY>::type_spec() }
            })
            .collect()
    }

    pub(crate) fn supplied_specs(&self) -> Vec<Tokens> {
        self.exposed_params
            .iter()
            .map(|param| {
                let index = param.rust_index;
                quote! { args[#index].clone() }
            })
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
        collect_usage(ty, declared, &mut visible, &mut error_only, Usage::Visible);
    } else {
        let mut visit_field = |field: &Field| -> syn::Result<()> {
            let attr = options(&field.attrs)?;
            if let FieldProjection::Rust(ty) = project_field(field, &attr) {
                collect_usage(ty, declared, &mut visible, &mut error_only, Usage::Visible);
            }
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

#[derive(Clone, Copy)]
enum Usage {
    Visible,
    Error,
}

fn collect_usage(
    ty: &Type,
    declared: &HashSet<String>,
    visible: &mut HashSet<String>,
    error_only: &mut HashSet<String>,
    usage: Usage,
) {
    if matches!(usage, Usage::Visible) {
        if let Some((ok, err)) = result_args(ty) {
            collect_usage(ok, declared, visible, error_only, Usage::Visible);
            collect_usage(err, declared, visible, error_only, Usage::Error);
            return;
        }
    }
    if let Some(param) = is_param(ty, declared) {
        let target = match usage {
            Usage::Visible => &mut *visible,
            Usage::Error => &mut *error_only,
        };
        target.insert(param);
    }
    for arg in type_args(ty) {
        collect_usage(arg, declared, visible, error_only, usage);
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
