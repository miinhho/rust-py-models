use crate::python::python_class_ident;
use proc_macro2::TokenStream as Tokens;
use quote::quote;
use syn::{GenericParam, Generics, Ident};

pub(crate) struct GenericInfo {
    pub(crate) impl_generics: Generics,
    pub(crate) names: Vec<String>,
    type_params: Vec<Ident>,
    has_params: bool,
}

impl GenericInfo {
    pub(crate) fn parse(original: &Generics) -> syn::Result<Self> {
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
        let mut impl_generics = original.clone();
        let type_params = original
            .type_params()
            .map(|param| param.ident.clone())
            .collect::<Vec<_>>();
        for param in &type_params {
            python_class_ident(&param.to_string(), param.span())?;
            impl_generics
                .make_where_clause()
                .predicates
                .push(syn::parse_quote!(#param: ::py_rs::PY));
        }
        let names = type_params.iter().map(ToString::to_string).collect();
        Ok(Self {
            impl_generics,
            names,
            type_params,
            has_params: !original.params.is_empty(),
        })
    }

    pub(crate) fn has_params(&self) -> bool {
        self.has_params
    }

    pub(crate) fn typevar_definitions(&self) -> Vec<String> {
        self.names
            .iter()
            .map(|param| format!("{param} = TypeVar(\"{param}\")"))
            .collect()
    }

    pub(crate) fn inline(&self, name: &str) -> (Tokens, Tokens) {
        if self.type_params.is_empty() {
            return (
                quote! { String::from(#name) },
                quote! { String::from(#name) },
            );
        }
        let concrete_args = self
            .type_params
            .iter()
            .map(|param| quote! { <#param as ::py_rs::PY>::inline() });
        (
            quote! { format!("{}[{}]", #name, [#(#concrete_args),*].join(", ")) },
            quote! { format!("{}[{}]", #name, args.join(", ")) },
        )
    }
}
