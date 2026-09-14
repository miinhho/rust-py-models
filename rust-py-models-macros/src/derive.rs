use crate::attrs::{options, Options};
use crate::generics::GenericInfo;
use crate::model;
use crate::python::python_class_ident;
use proc_macro2::TokenStream as Tokens;
use quote::quote;
use std::collections::HashSet;
use syn::{Data, DeriveInput, Generics, Ident};

pub(crate) fn expand(input: DeriveInput) -> syn::Result<Tokens> {
    let mut attr = options(&input.attrs)?;
    let generic = GenericInfo::parse(&input.generics, &attr, &input.data)?;
    if let Some(as_type) = attr.as_type.take() {
        attr.as_type = Some(generic.concretize_type(as_type));
    }
    validate_container(&input.ident, &input.generics, &attr)?;
    let ident = input.ident;
    let name = attr
        .effective_rename()
        .map_or_else(|| ident.to_string(), str::to_owned);
    python_class_ident(&name, ident.span())?;
    let output = attr
        .export_to
        .clone()
        .unwrap_or_else(|| format!("{name}.py"));

    let implementation = if let Some(as_type) = &attr.as_type {
        alias_impl(&ident, as_type, &attr, &generic)?
    } else {
        model_impl(&ident, input.data, &name, &output, &attr, &generic)?
    };
    let registration = registration(&ident, attr.export);
    Ok(quote! {
        #implementation
        #registration
    })
}

fn validate_container(ident: &Ident, generics: &Generics, attr: &Options) -> syn::Result<()> {
    if attr.unsafe_python_type.is_some() || attr.import.is_some() {
        return Err(syn::Error::new_spanned(
            ident,
            "type overrides belong on fields",
        ));
    }
    if attr.newtype && attr.as_type.is_some() {
        return Err(syn::Error::new_spanned(
            ident,
            "#[py(newtype)] cannot be combined with #[py(as = \"...\")]",
        ));
    }
    if attr.is_skipped() {
        return Err(syn::Error::new(
            ident.span(),
            "#[py(skip)] belongs on a field or enum variant",
        ));
    }
    if attr.export && !generics.params.is_empty() {
        return Err(syn::Error::new_spanned(
            ident,
            "#[py(export)] on a generic type needs concrete type arguments; export a concrete instantiation explicitly",
        ));
    }
    Ok(())
}

fn registration(ident: &Ident, export: bool) -> Tokens {
    if export {
        quote! {
            ::rust_py_models::__private::inventory::submit! {
                ::rust_py_models::__private::ExportRoot::new(
                    concat!(module_path!(), "::", stringify!(#ident)),
                    |dir| <#ident as ::rust_py_models::PY>::export_all_to(dir),
                )
            }
        }
    } else {
        quote! {}
    }
}

fn alias_impl(
    ident: &Ident,
    as_type: &syn::Type,
    attr: &Options,
    generic: &GenericInfo,
) -> syn::Result<Tokens> {
    if attr.export {
        return Err(syn::Error::new_spanned(
            ident,
            "#[py(export)] cannot be combined with #[py(as = \"...\")]",
        ));
    }
    let (impl_generics, ty_generics, where_clause) = generic.impl_generics.split_for_impl();
    let params = generic.names.iter().cloned().collect::<HashSet<_>>();
    let concrete = crate::type_expr::concrete(as_type, &params);
    let supplied = crate::type_expr::supplied(as_type, &generic.all_names);
    Ok(quote! {
        impl #impl_generics ::rust_py_models::PY for #ident #ty_generics #where_clause {
            fn type_spec() -> ::rust_py_models::TypeSpec { #concrete }

            fn type_spec_with(args: &[::rust_py_models::TypeSpec])
                -> ::rust_py_models::TypeSpec
            {
                #supplied
            }
        }
    })
}

fn model_pieces(
    ident: &Ident,
    data: Data,
    name: &str,
    attr: &Options,
    generic: &GenericInfo,
) -> syn::Result<model::ModelPieces> {
    match data {
        Data::Struct(data) => {
            let data = generic.concretize_struct(data);
            model::structure(data, name, attr, &generic.names)
        }
        Data::Enum(data) => {
            let data = generic.concretize_enum(data);
            model::enumeration(data, name, ident.span(), attr, &generic.names)
        }
        Data::Union(data) => Err(syn::Error::new_spanned(
            data.union_token,
            "PY cannot be derived for unions",
        )),
    }
}

fn model_impl(
    ident: &Ident,
    data: Data,
    name: &str,
    output: &str,
    attr: &Options,
    generic: &GenericInfo,
) -> syn::Result<Tokens> {
    let model::ModelPieces {
        declarations,
        hashable,
    } = model_pieces(ident, data, name, attr, generic)?;
    let (impl_generics, ty_generics, where_clause) = generic.impl_generics.split_for_impl();
    let concrete_specs = generic.concrete_specs();
    let supplied_specs = generic.supplied_specs();
    let typevars = &generic.names;
    Ok(quote! {
        impl #impl_generics ::rust_py_models::PY for #ident #ty_generics #where_clause {
            fn type_spec() -> ::rust_py_models::TypeSpec {
                let hashable = ::rust_py_models::__private::check_hashability::<Self>(|| #hashable);
                ::rust_py_models::TypeSpec::model(
                    #name,
                    vec![#(#concrete_specs),*],
                    ::rust_py_models::Dependency::of::<Self>(
                        concat!(module_path!(), "::", stringify!(#ident)),
                        #name,
                        #output,
                    ),
                    hashable,
                )
            }

            fn type_spec_with(args: &[::rust_py_models::TypeSpec])
                -> ::rust_py_models::TypeSpec
            {
                let hashable = ::rust_py_models::__private::check_hashability::<Self>(|| #hashable);
                ::rust_py_models::TypeSpec::model(
                    #name,
                    vec![#(#supplied_specs),*],
                    ::rust_py_models::Dependency::of::<Self>(
                        concat!(module_path!(), "::", stringify!(#ident)),
                        #name,
                        #output,
                    ),
                    hashable,
                )
            }

            fn model_spec()
                -> Result<Option<::rust_py_models::ModelSpec>, ::rust_py_models::ExportError>
            {
                let hashable = ::rust_py_models::__private::check_hashability::<Self>(|| #hashable);
                let mut model = ::rust_py_models::ModelSpec::new::<Self>(
                    concat!(module_path!(), "::", stringify!(#ident)),
                    #name,
                    #output,
                    hashable,
                );
                #(
                    model.push(::rust_py_models::Declaration::TypeVar(String::from(#typevars)));
                )*
                #(model.push(#declarations);)*
                Ok(Some(model))
            }
        }
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
    fn accepts_as_override() {
        let input: DeriveInput = syn::parse_quote! {
            struct User { #[py(as = "String")] name: u64 }
        };
        expand(input).unwrap();
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
            .contains("requires #[py(unsafe_type = ...)]"));

        let duplicate: DeriveInput = syn::parse_quote! {
            struct Event { id: u64, #[py(rename = "id")] other: u64 }
        };
        assert!(expand(duplicate)
            .unwrap_err()
            .to_string()
            .contains("duplicate Python field name"));
    }

    #[test]
    fn newtype_requires_one_non_generic_tuple_field() {
        let valid: DeriveInput = syn::parse_quote! {
            #[py(newtype)]
            struct UserId(u64);
        };
        expand(valid).unwrap();

        let named: DeriveInput = syn::parse_quote! {
            #[py(newtype)]
            struct UserId { value: u64 }
        };
        assert!(expand(named)
            .unwrap_err()
            .to_string()
            .contains("requires a one-field tuple struct"));

        let generic: DeriveInput = syn::parse_quote! {
            #[py(newtype)]
            struct UserId<T>(T);
        };
        assert!(expand(generic)
            .unwrap_err()
            .to_string()
            .contains("generic #[py(newtype)] declarations are not supported"));
    }

    #[test]
    fn unsafe_type_name_is_explicit() {
        let valid: DeriveInput = syn::parse_quote! {
            struct Price {
                #[py(unsafe_type = "decimal.Decimal", import = "decimal")]
                value: u64,
            }
        };
        expand(valid).unwrap();

        let old_name: DeriveInput = syn::parse_quote! {
            struct Price {
                #[py(type = "decimal.Decimal")]
                value: u64,
            }
        };
        assert!(expand(old_name)
            .unwrap_err()
            .to_string()
            .contains("unsupported #[py(...)] option"));
    }
}
