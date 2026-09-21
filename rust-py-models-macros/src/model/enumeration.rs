use super::{dataclass_options, documentation, ModelPieces};
use crate::attrs::{options, DataclassOptions, Options, RenameRule};
use crate::fields::{fields, FieldPieces};
use crate::model::tagging::Tagging;
use crate::python::{python_class_ident, python_ident};
use proc_macro2::{Span, TokenStream as Tokens};
use quote::quote;
use std::collections::HashSet;
use syn::{DataEnum, Fields, Variant};

pub(super) fn render(
    data: DataEnum,
    name: &str,
    span: Span,
    container: &Options,
    params: &[String],
    projection: &crate::projection::ProjectionMap,
) -> syn::Result<ModelPieces> {
    let had_payload = data
        .variants
        .iter()
        .any(|variant| !matches!(variant.fields, Fields::Unit));
    let dataclass = container.dataclass;
    let rename_all = container.serde_rename_all;
    let container_documentation = documentation(&container.documentation);
    let tagging = Tagging::from_options(container, span)?;
    let mut variants = Vec::new();
    for variant in data.variants {
        let attr = options(&variant.attrs)?;
        validate_variant_options(&variant, &attr)?;
        if !attr.is_skipped() {
            variants.push((variant, attr));
        }
    }
    if variants.is_empty() && (had_payload || !matches!(&tagging, Tagging::External)) {
        return Err(syn::Error::new(
            span,
            "enum has no variants that can be represented in Python",
        ));
    }
    let all_unit = variants
        .iter()
        .all(|(variant, _)| matches!(variant.fields, Fields::Unit));
    if all_unit && matches!(tagging, Tagging::External) {
        render_unit(
            variants,
            name,
            span,
            dataclass,
            rename_all,
            container_documentation,
        )
    } else if all_unit && matches!(tagging, Tagging::Untagged) {
        Err(syn::Error::new(
            span,
            "serde untagged unit-only enums are not supported",
        ))
    } else {
        render_payload(
            variants,
            name,
            container,
            tagging,
            params,
            container_documentation,
            projection,
        )
    }
}

fn validate_variant_options(variant: &Variant, attr: &Options) -> syn::Result<()> {
    if attr.serde_rename_all.is_some()
        || attr.serde_rename_all_fields.is_some()
        || attr.serde_tag.is_some()
        || attr.serde_content.is_some()
        || attr.serde_untagged
        || attr.serde_flatten
    {
        return Err(syn::Error::new_spanned(
            variant,
            "container-only serde attributes cannot be used on an enum variant",
        ));
    }
    if attr.export
        || attr.export_to.is_some()
        || attr.unsafe_python_type.is_some()
        || attr.import.is_some()
        || attr.newtype
    {
        return Err(syn::Error::new_spanned(
            variant,
            "export and type override options do not apply to enum variants",
        ));
    }
    Ok(())
}

fn render_unit(
    variants: Vec<(Variant, Options)>,
    name: &str,
    span: Span,
    dataclass: DataclassOptions,
    rename_all: Option<RenameRule>,
    container_documentation: Tokens,
) -> syn::Result<ModelPieces> {
    if dataclass.is_set() {
        return Err(syn::Error::new(
            span,
            "dataclass options do not apply to a unit enum",
        ));
    }
    let mut names = HashSet::new();
    let mut members = Vec::new();
    for (variant, attr) in variants {
        if attr.dataclass.is_set() {
            return Err(syn::Error::new_spanned(
                variant,
                "dataclass options do not apply to a unit enum",
            ));
        }
        let ident = rust_variant_name(&variant)?;
        let value = attr.effective_rename().map_or_else(
            || rename_all.map_or_else(|| ident.clone(), |rule| rule.apply(&ident)),
            str::to_owned,
        );
        if !names.insert(value.clone()) {
            return Err(syn::Error::new_spanned(
                variant,
                format!("duplicate Python enum value {value:?}"),
            ));
        }
        let member_documentation = documentation(&attr.documentation);
        members.push(quote! {
            ::rust_py_models::EnumMemberSpec {
                name: String::from(#ident),
                value: String::from(#value),
                documentation: #member_documentation,
            }
        });
    }
    Ok(ModelPieces {
        declarations: vec![quote! {
            ::rust_py_models::Declaration::StringEnum(
                ::rust_py_models::StringEnumSpec {
                    name: String::from(#name),
                    members: vec![#(#members),*],
                    documentation: #container_documentation,
                },
            )
        }],
        hashable: quote! { true },
    })
}

fn render_payload(
    variants: Vec<(Variant, Options)>,
    name: &str,
    container: &Options,
    tagging: Tagging,
    params: &[String],
    container_documentation: Tokens,
    projection: &crate::projection::ProjectionMap,
) -> syn::Result<ModelPieces> {
    let dataclass = container.dataclass;
    let rename_all = container.serde_rename_all;
    let rename_all_fields = container.serde_rename_all_fields;
    let mut declarations = Vec::new();
    let mut alias_names = Vec::new();
    let mut hash_checks = Vec::new();
    let mut class_names = HashSet::new();
    let param_set = params.iter().cloned().collect();

    for (variant, attr) in variants {
        let ident = rust_variant_name(&variant)?;
        let suffix = attr.rename.clone().unwrap_or_else(|| ident.clone());
        python_ident(&suffix, variant.ident.span())?;
        let class_name = format!("{name}{suffix}");
        python_class_ident(&class_name, variant.ident.span())?;
        if !class_names.insert(class_name.clone()) {
            return Err(syn::Error::new_spanned(
                &variant,
                format!("duplicate Python variant class {class_name:?}"),
            ));
        }
        let serialized_name = attr
            .serde_rename
            .clone()
            .unwrap_or_else(|| rename_all.map_or_else(|| ident.clone(), |rule| rule.apply(&ident)));
        let variant_options = dataclass.with_overrides(attr.dataclass);
        let options = dataclass_options(variant_options);
        let variant_documentation = documentation(&attr.documentation);
        let FieldPieces {
            statements,
            specs,
            hash_checks: field_hashes,
            ..
        } = fields(&variant.fields, &param_set, rename_all_fields, projection)?;
        let frozen = variant_options.frozen.unwrap_or(false);
        hash_checks.push(quote! { #frozen #(&& #field_hashes)* });
        alias_names.push(class_name.clone());

        match &tagging {
            Tagging::External | Tagging::Untagged => {
                declarations.push(dataclass_declaration(
                    &class_name,
                    variant_documentation.clone(),
                    options,
                    params,
                    statements,
                ));
            }
            Tagging::Internal(tag) => {
                if matches!(variant.fields, Fields::Unnamed(_)) {
                    return Err(syn::Error::new_spanned(
                        variant,
                        "internally tagged enums require unit or named variants",
                    ));
                }
                let discriminator = discriminator(tag, &serialized_name);
                let mut tagged = vec![discriminator];
                tagged.extend(statements);
                declarations.push(dataclass_declaration(
                    &class_name,
                    variant_documentation.clone(),
                    options,
                    params,
                    tagged,
                ));
            }
            Tagging::Adjacent { tag, content } => {
                let discriminator = discriminator(tag, &serialized_name);
                match &variant.fields {
                    Fields::Unit => declarations.push(dataclass_declaration(
                        &class_name,
                        variant_documentation.clone(),
                        options,
                        params,
                        vec![discriminator],
                    )),
                    Fields::Unnamed(_) => {
                        let content_spec = if let [only] = specs.as_slice() {
                            only.clone()
                        } else {
                            quote! { ::rust_py_models::TypeSpec::tuple(vec![#(#specs),*]) }
                        };
                        let content_field = quote! {
                            fields.push(::rust_py_models::FieldSpec::new(#content, #content_spec));
                        };
                        declarations.push(dataclass_declaration(
                            &class_name,
                            variant_documentation.clone(),
                            options,
                            params,
                            vec![discriminator, content_field],
                        ));
                    }
                    Fields::Named(_) => {
                        let content_name = format!("{class_name}Content");
                        python_class_ident(&content_name, variant.ident.span())?;
                        if !class_names.insert(content_name.clone()) {
                            return Err(syn::Error::new_spanned(
                                &variant,
                                format!("duplicate Python content class {content_name:?}"),
                            ));
                        }
                        declarations.push(dataclass_declaration(
                            &content_name,
                            quote! { None },
                            options.clone(),
                            params,
                            statements,
                        ));
                        let content_type = local_generic_type(&content_name, params);
                        let content_field = quote! {
                            fields.push(::rust_py_models::FieldSpec::new(#content, #content_type));
                        };
                        declarations.push(dataclass_declaration(
                            &class_name,
                            variant_documentation,
                            options,
                            params,
                            vec![discriminator, content_field],
                        ));
                    }
                }
            }
        }
    }

    let variants = alias_names
        .iter()
        .map(|class_name| local_generic_type(class_name, params))
        .collect::<Vec<_>>();
    declarations.push(quote! {
        ::rust_py_models::Declaration::TypeAlias {
            name: String::from(#name),
            target: ::rust_py_models::TypeSpec::union(vec![#(#variants),*]),
            documentation: #container_documentation,
        }
    });
    Ok(ModelPieces {
        declarations,
        hashable: quote! { true #(&& #hash_checks)* },
    })
}

fn dataclass_declaration(
    name: &str,
    documentation: Tokens,
    options: Tokens,
    params: &[String],
    statements: Vec<Tokens>,
) -> Tokens {
    quote! {
        {
            let mut fields = Vec::new();
            #(#statements)*
            ::rust_py_models::Declaration::Dataclass(
                ::rust_py_models::DataclassSpec {
                    name: String::from(#name),
                    documentation: #documentation,
                    options: #options,
                    type_params: vec![#(String::from(#params)),*],
                    fields,
                },
            )
        }
    }
}

fn local_generic_type(name: &str, params: &[String]) -> Tokens {
    if params.is_empty() {
        quote! { ::rust_py_models::TypeSpec::named(#name) }
    } else {
        quote! {
            ::rust_py_models::TypeSpec::subscript(
                ::rust_py_models::TypeSpec::named(#name),
                vec![#(::rust_py_models::TypeSpec::named(#params)),*],
            )
        }
    }
}

fn discriminator(tag: &str, value: &str) -> Tokens {
    quote! {
        fields.push(
            ::rust_py_models::FieldSpec::new(
                #tag,
                ::rust_py_models::TypeSpec::literal(#value),
            )
            .with_default(::rust_py_models::FieldDefault::DataclassField {
                init: false,
                value: String::from(#value),
            }),
        );
    }
}

fn rust_variant_name(variant: &Variant) -> syn::Result<String> {
    let ident = variant
        .ident
        .to_string()
        .trim_start_matches("r#")
        .to_owned();
    python_ident(&ident, variant.ident.span())?;
    Ok(ident)
}
