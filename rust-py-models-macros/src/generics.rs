use crate::attrs::{options, Options};
use crate::fields::{project_field, FieldProjection};
use crate::projection::ProjectionMap;
use crate::python::python_class_ident;
use crate::type_expr::{is_param, result_args};
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
    pub(crate) fn parse(
        original: &Generics,
        options: &Options,
        data: &Data,
        projection: &ProjectionMap,
        projected_names: Option<&HashSet<String>>,
    ) -> syn::Result<Self> {
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

        let visible = if let Some(names) = projected_names {
            names.clone()
        } else {
            visible_params(data, options, &declared, projection)?
        };
        let mut impl_generics = original.clone();
        let exposed_params = original
            .type_params()
            .enumerate()
            .filter(|(index, _)| {
                !concrete.contains_key(&all_names[*index]) && visible.contains(&all_names[*index])
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

pub(crate) fn visible_params(
    data: &Data,
    container: &Options,
    declared: &HashSet<String>,
    projection: &ProjectionMap,
) -> syn::Result<HashSet<String>> {
    let mut visible = HashSet::new();
    if let Some(ty) = &container.as_type {
        collect_usage(ty, declared, projection, &mut visible);
    } else {
        let mut visit_field = |field: &Field| -> syn::Result<()> {
            let attr = options(&field.attrs)?;
            if let FieldProjection::Rust(ty) = project_field(field, &attr) {
                collect_usage(ty, declared, projection, &mut visible);
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
    Ok(visible)
}

fn collect_usage(
    ty: &Type,
    declared: &HashSet<String>,
    projection: &ProjectionMap,
    visible: &mut HashSet<String>,
) {
    if let Some((ok, _)) = result_args(ty) {
        collect_usage(ok, declared, projection, visible);
        return;
    }
    if let Some(param) = is_param(ty, declared) {
        visible.insert(param);
    }
    for (_, arg) in projection.args(ty).into_iter().filter(|(used, _)| *used) {
        collect_usage(arg, declared, projection, visible);
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
