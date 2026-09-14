use crate::attrs::{options, Options, RenameRule};
use crate::projection::ProjectionMap;
use crate::python::python_ident;
use proc_macro2::TokenStream as Tokens;
use quote::quote;
use std::collections::HashSet;
use syn::{Field, Fields, Type};

pub(crate) enum FieldProjection<'a> {
    Omitted,
    Unsafe {
        annotation: &'a str,
        import: Option<&'a str>,
    },
    Rust(&'a Type),
}

pub(crate) fn project_field<'a>(field: &'a Field, attr: &'a Options) -> FieldProjection<'a> {
    if attr.is_skipped()
        || (attr.unsafe_python_type.is_none()
            && attr.as_type.is_none()
            && is_phantom_data(&field.ty))
    {
        FieldProjection::Omitted
    } else if let Some(annotation) = attr.unsafe_python_type.as_deref() {
        FieldProjection::Unsafe {
            annotation,
            import: attr.import.as_deref(),
        }
    } else {
        FieldProjection::Rust(attr.as_type.as_ref().unwrap_or(&field.ty))
    }
}

#[derive(Default)]
pub(crate) struct FieldPieces {
    pub(crate) statements: Vec<Tokens>,
    pub(crate) specs: Vec<Tokens>,
    pub(crate) hash_checks: Vec<Tokens>,
    pub(crate) empty: bool,
}

pub(crate) fn fields(
    fields: &Fields,
    params: &HashSet<String>,
    rename_all: Option<RenameRule>,
    mappings: &ProjectionMap,
) -> syn::Result<FieldPieces> {
    let mut out = FieldPieces::default();
    let mut names = HashSet::new();
    for (index, field) in fields.iter().enumerate() {
        let attr = options(&field.attrs)?;
        validate_options(field, &attr)?;
        let projection = project_field(field, &attr);
        if matches!(projection, FieldProjection::Omitted) {
            continue;
        }
        if add_flattened_field(field, &attr, &mut out)? {
            continue;
        }
        let name = field_name(field, index, &attr, rename_all);
        python_ident(
            &name,
            field
                .ident
                .as_ref()
                .map_or_else(proc_macro2::Span::call_site, proc_macro2::Ident::span),
        )?;
        if name.starts_with("__") && name.ends_with("__") {
            return Err(syn::Error::new_spanned(
                field,
                "Python dunder names cannot be dataclass fields",
            ));
        }
        if !names.insert(name.clone()) {
            return Err(syn::Error::new_spanned(
                field,
                format!("duplicate Python field name {name:?}"),
            ));
        }
        let documentation = option_string(&attr.documentation);

        let (spec, hash_check) = match projection {
            FieldProjection::Unsafe { annotation, import } => {
                if annotation.trim().is_empty() {
                    return Err(syn::Error::new_spanned(
                        field,
                        "empty unsafe Python type override",
                    ));
                }
                if let Some(module) = import {
                    for part in module.split('.') {
                        python_ident(part, proc_macro2::Span::call_site())?;
                    }
                }
                let spec = if let Some(module) = import {
                    quote! {
                        ::rust_py_models::TypeSpec::unsafe_raw(#annotation).with_import(#module)
                    }
                } else {
                    quote! { ::rust_py_models::TypeSpec::unsafe_raw(#annotation) }
                };
                (spec, quote! { false })
            }
            FieldProjection::Rust(ty) => (
                crate::type_expr::symbolic(ty, params, mappings),
                quote! { <#ty as ::rust_py_models::PY>::is_hashable() },
            ),
            FieldProjection::Omitted => unreachable!("omitted fields are skipped above"),
        };
        out.statements.push(quote! {
            fields.push(
                ::rust_py_models::FieldSpec::new(#name, #spec)
                    .with_documentation(#documentation)
            );
        });
        out.specs.push(spec);
        out.hash_checks.push(hash_check);
    }
    out.empty = out.statements.is_empty();
    Ok(out)
}

fn validate_options(field: &Field, attr: &Options) -> syn::Result<()> {
    if attr.serde_rename_all.is_some()
        || attr.serde_rename_all_fields.is_some()
        || attr.serde_tag.is_some()
        || attr.serde_content.is_some()
        || attr.serde_untagged
    {
        return Err(syn::Error::new_spanned(
            field,
            "container-only serde attributes cannot be used on a field",
        ));
    }
    if attr.export || attr.export_to.is_some() || attr.dataclass.is_set() || attr.newtype {
        return Err(syn::Error::new_spanned(
            field,
            "dataclass and export options belong on the struct or enum variant",
        ));
    }
    if !attr.concrete.is_empty() || attr.bound.is_some() {
        return Err(syn::Error::new_spanned(
            field,
            "generic concrete and bound options belong on a struct or enum",
        ));
    }
    if attr.as_type.is_some() && attr.unsafe_python_type.is_some() {
        return Err(syn::Error::new_spanned(
            field,
            "#[py(as = ...)] cannot be combined with #[py(unsafe_type = ...)]",
        ));
    }
    if attr.import.is_some() && attr.unsafe_python_type.is_none() {
        return Err(syn::Error::new_spanned(
            field,
            "#[py(import = ...)] requires #[py(unsafe_type = ...)]",
        ));
    }
    Ok(())
}

fn add_flattened_field(field: &Field, attr: &Options, out: &mut FieldPieces) -> syn::Result<bool> {
    if !attr.serde_flatten {
        return Ok(false);
    }
    if field.ident.is_none() {
        return Err(syn::Error::new_spanned(
            field,
            "serde flatten cannot be used on a tuple field",
        ));
    }
    if attr.effective_rename().is_some()
        || attr.unsafe_python_type.is_some()
        || attr.as_type.is_some()
        || attr.import.is_some()
    {
        return Err(syn::Error::new_spanned(
            field,
            "serde flatten is incompatible with rename and type overrides",
        ));
    }
    let ty = &field.ty;
    out.statements.push(quote! {
        let nested = <#ty as ::rust_py_models::PY>::model_spec()?
            .ok_or(::rust_py_models::ExportError::NotFlattenable(
                ::std::any::type_name::<#ty>(),
            ))?;
        fields.extend(nested.flattened_fields()?);
    });
    out.hash_checks.push(quote! {
        <#ty as ::rust_py_models::PY>::model_spec()
            .ok()
            .flatten()
            .and_then(|model| model.flattened_fields().ok())
            .is_some_and(|fields| fields.iter().all(|field| field.ty.is_hashable()))
    });
    Ok(true)
}

fn field_name(
    field: &Field,
    index: usize,
    attr: &Options,
    rename_all: Option<RenameRule>,
) -> String {
    attr.effective_rename().map_or_else(
        || {
            field.ident.as_ref().map_or_else(
                || format!("_{index}"),
                |ident| {
                    let rust_name = ident.to_string();
                    let rust_name = rust_name.trim_start_matches("r#");
                    rename_all.map_or_else(|| rust_name.to_owned(), |rule| rule.apply(rust_name))
                },
            )
        },
        str::to_owned,
    )
}

fn is_phantom_data(ty: &syn::Type) -> bool {
    let syn::Type::Path(path) = ty else {
        return false;
    };
    let names = path
        .path
        .segments
        .iter()
        .map(|segment| segment.ident.to_string())
        .collect::<Vec<_>>();
    matches!(names.as_slice(), [one] if one == "PhantomData")
        || matches!(names.as_slice(), [root, marker, phantom] if (root == "std" || root == "core") && marker == "marker" && phantom == "PhantomData")
}

fn option_string(value: &Option<String>) -> Tokens {
    value.as_ref().map_or_else(
        || quote! { None },
        |value| quote! { Some(String::from(#value)) },
    )
}
