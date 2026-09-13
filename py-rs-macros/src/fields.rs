use crate::attrs::options;
use crate::python::python_ident;
use proc_macro2::TokenStream as Tokens;
use quote::quote;
use std::collections::HashSet;
use syn::Fields;

#[derive(Default)]
pub(crate) struct FieldPieces {
    pub(crate) lines: Vec<Tokens>,
    pub(crate) deps: Vec<Tokens>,
    pub(crate) imports: Vec<Tokens>,
    pub(crate) definitions: Vec<Tokens>,
    pub(crate) hash_checks: Vec<Tokens>,
    pub(crate) validations: Vec<Tokens>,
    pub(crate) empty: bool,
}

pub(crate) fn fields(fields: &Fields, params: &HashSet<String>) -> syn::Result<FieldPieces> {
    let mut out = FieldPieces::default();
    let mut names = std::collections::HashSet::new();
    for (index, field) in fields.iter().enumerate() {
        let attr = options(&field.attrs)?;
        if attr.export || attr.export_to.is_some() || attr.dataclass.is_set() {
            return Err(syn::Error::new_spanned(
                field,
                "dataclass and export options belong on the struct or enum variant",
            ));
        }
        if attr.import.is_some() && attr.python_type.is_none() {
            return Err(syn::Error::new_spanned(
                field,
                "#[py(import = ...)] requires #[py(type = ...)]",
            ));
        }
        if attr.skip || (attr.python_type.is_none() && is_phantom_data(&field.ty)) {
            continue;
        }
        let name = attr.rename.unwrap_or_else(|| {
            field
                .ident
                .as_ref()
                .map(|i| i.to_string().trim_start_matches("r#").to_owned())
                .unwrap_or_else(|| format!("_{index}"))
        });
        python_ident(
            &name,
            field
                .ident
                .as_ref()
                .map(|i| i.span())
                .unwrap_or_else(proc_macro2::Span::call_site),
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
        if let Some(python_type) = attr.python_type {
            if python_type.trim().is_empty() {
                return Err(syn::Error::new_spanned(field, "empty Python type override"));
            }
            out.lines.push(quote! {
                out.push_str("    ");
                out.push_str(#name);
                out.push_str(": ");
                out.push_str(#python_type);
                out.push('\n');
            });
            if let Some(module) = attr.import {
                for part in module.split('.') {
                    python_ident(part, proc_macro2::Span::call_site())?;
                }
                let statement = format!("import {module}");
                out.imports
                    .push(quote! { imports.push(#statement.into()); });
            }
            out.hash_checks.push(quote! { false });
            continue;
        }
        let ty = &field.ty;
        let inline = crate::type_expr::inline(ty, params);
        out.lines.push(quote! {
            out.push_str("    ");
            out.push_str(#name);
            out.push_str(": ");
            out.push_str(&#inline);
            out.push('\n');
        });
        out.deps.extend(crate::type_expr::dependencies(ty, params));
        out.imports.extend(crate::type_expr::imports(ty, params));
        out.definitions
            .extend(crate::type_expr::definitions(ty, params));
        out.hash_checks
            .push(quote! { <#ty as ::py_rs::PY>::is_hashable() });
        out.validations
            .push(quote! { <#ty as ::py_rs::PY>::annotation_validate()?; });
    }
    out.empty = out.lines.is_empty();
    Ok(out)
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
