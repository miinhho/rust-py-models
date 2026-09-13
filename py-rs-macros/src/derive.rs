use crate::attrs::options;
use crate::generics::GenericInfo;
use crate::model;
use crate::python::python_class_ident;
use proc_macro2::TokenStream as Tokens;
use quote::{format_ident, quote};
use syn::{Data, DeriveInput};

pub(crate) fn expand(input: DeriveInput) -> syn::Result<Tokens> {
    let generic = GenericInfo::parse(&input.generics)?;
    let (impl_generics, ty_generics, where_clause) = generic.impl_generics.split_for_impl();
    let ident = input.ident;
    let attr = options(&input.attrs)?;
    let dataclass = attr.dataclass;
    if attr.python_type.is_some() || attr.import.is_some() {
        return Err(syn::Error::new_spanned(
            &ident,
            "type overrides belong on fields",
        ));
    }
    if attr.skip {
        return Err(syn::Error::new(
            ident.span(),
            "#[py(skip)] belongs on a field or enum variant",
        ));
    }
    let name = attr.rename.unwrap_or_else(|| ident.to_string());
    python_class_ident(&name, ident.span())?;
    let output = attr.export_to.unwrap_or_else(|| format!("{name}.py"));
    let model::ModelPieces {
        prelude,
        declaration,
        deps,
        imports,
        definitions,
        validations,
        hashable,
        local_names,
    } = match input.data {
        Data::Struct(data) => model::structure(data, &name, dataclass, &generic.names),
        Data::Enum(data) => {
            model::enumeration(data, &name, ident.span(), dataclass, &generic.names)
        }
        Data::Union(data) => Err(syn::Error::new_spanned(
            data.union_token,
            "PY cannot be derived for unions",
        )),
    }?;
    if attr.export && generic.has_params() {
        return Err(syn::Error::new_spanned(&ident, "#[py(export)] on a generic type needs concrete type arguments; call export_all() on an instantiation instead"));
    }
    let test = if attr.export {
        let test_ident = format_ident!("export_bindings_{}", ident);
        quote! {
            #[test]
            #[allow(non_snake_case)]
            fn #test_ident() {
                <#ident as ::py_rs::PY>::export_all().expect("failed to export Python bindings");
            }
        }
    } else {
        quote! {}
    };
    let typevars = generic.typevar_definitions();
    let (generic_inline, symbolic_inline) = generic.inline(&name);
    Ok(quote! {
        impl #impl_generics ::py_rs::PY for #ident #ty_generics #where_clause {
            fn declaration_id() -> &'static str {
                concat!(module_path!(), "::", stringify!(#ident))
            }
            fn name() -> String { #name.into() }
            fn inline() -> String { #generic_inline }
            fn inline_with(args: &[String]) -> String { #symbolic_inline }
            fn prelude() -> String { #prelude }
            fn imports() -> Vec<String> {
                let mut imports = Vec::new();
                #(#imports)*
                imports
            }
            fn definitions() -> Vec<String> {
                let mut definitions = Vec::new();
                #(definitions.push(String::from(#typevars));)*
                #(#definitions)*
                definitions
            }
            fn is_hashable() -> bool {
                ::py_rs::__private::check_hashability::<Self>(|| #hashable)
            }
            fn validate() -> Result<(), ::py_rs::ExportError> {
                #(#validations)*
                Ok(())
            }
            fn decl() -> String { #declaration }
            fn dependencies() -> Vec<::py_rs::Dependency> {
                let mut deps = Vec::new();
                #(#deps)*
                deps
            }
            fn output_path() -> Option<::std::path::PathBuf> { Some(#output.into()) }
            fn declaration_names() -> Vec<String> {
                vec![#(String::from(#local_names)),*]
            }
        }
        #test
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_dataclass_options_on_unit_enums() {
        let input: DeriveInput = syn::parse_quote! {
            #[py(frozen)]
            enum Status { Active }
        };
        assert!(expand(input)
            .unwrap_err()
            .to_string()
            .contains("dataclass options do not apply to a unit enum"));
    }

    #[test]
    fn rejects_dataclass_options_on_fields() {
        let input: DeriveInput = syn::parse_quote! {
            struct User { #[py(slots)] id: u64 }
        };
        assert!(expand(input)
            .unwrap_err()
            .to_string()
            .contains("dataclass and export options belong"));
    }

    #[test]
    fn raw_payload_variant_uses_python_class_name_in_alias() {
        let input: DeriveInput = syn::parse_quote! {
            enum Event { r#type(u64) }
        };
        let tokens = expand(input).unwrap().to_string();
        assert!(tokens.contains("Eventtype"));
        assert!(!tokens.contains("Eventr#type"));
    }

    #[test]
    fn rejects_names_reserved_by_optional_type_annotations() {
        for name in ["bytes", "object"] {
            assert!(
                crate::python::python_class_ident(name, proc_macro2::Span::call_site()).is_err()
            );
        }
    }

    #[test]
    fn rejects_duplicate_payload_enum_class_names() {
        let input: DeriveInput = syn::parse_quote! {
            enum Event {
                Login { user_id: i32 },
                #[py(rename = "Login")]
                Logout { user_id: i32 },
            }
        };
        assert!(expand(input)
            .unwrap_err()
            .to_string()
            .contains("duplicate Python variant class"));
    }

    #[test]
    fn rejects_as_override() {
        let input: DeriveInput = syn::parse_quote! {
            struct User { #[py(as = "String")] name: String }
        };
        assert!(expand(input)
            .unwrap_err()
            .to_string()
            .contains("unsupported #[py(...)] option"));
    }

    #[test]
    fn rejects_const_generics_and_implicit_generic_exports() {
        let const_generic: DeriveInput = syn::parse_quote! {
            struct Fixed<const N: usize> { value: [u8; N] }
        };
        assert!(expand(const_generic)
            .unwrap_err()
            .to_string()
            .contains("const generics are not supported"));

        let implicit_export: DeriveInput = syn::parse_quote! {
            #[py(export)]
            struct Page<T> { value: T }
        };
        assert!(expand(implicit_export)
            .unwrap_err()
            .to_string()
            .contains("needs concrete type arguments"));
    }

    #[test]
    fn rejects_orphaned_import_and_duplicate_field_names() {
        let orphaned_import: DeriveInput = syn::parse_quote! {
            struct Event { #[py(import = "decimal")] amount: u64 }
        };
        assert!(expand(orphaned_import)
            .unwrap_err()
            .to_string()
            .contains("requires #[py(type = ...)]"));

        let duplicate: DeriveInput = syn::parse_quote! {
            struct Event { id: u64, #[py(rename = "id")] other: u64 }
        };
        assert!(expand(duplicate)
            .unwrap_err()
            .to_string()
            .contains("duplicate Python field name"));
    }
}
