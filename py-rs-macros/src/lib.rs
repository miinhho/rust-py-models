use proc_macro::TokenStream;
use proc_macro2::TokenStream as Tokens;
use quote::{format_ident, quote};
use syn::{parse_macro_input, Attribute, Data, DeriveInput, Fields, LitStr};

#[derive(Default)]
struct Options {
    rename: Option<String>,
    skip: bool,
    export: bool,
    export_to: Option<String>,
}

fn options(attrs: &[Attribute]) -> syn::Result<Options> {
    let mut out = Options::default();
    for attr in attrs.iter().filter(|a| a.path().is_ident("py")) {
        attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("rename") {
                out.rename = Some(meta.value()?.parse::<LitStr>()?.value());
            } else if meta.path.is_ident("skip") {
                out.skip = true;
            } else if meta.path.is_ident("export") {
                out.export = true;
            } else if meta.path.is_ident("export_to") {
                out.export_to = Some(meta.value()?.parse::<LitStr>()?.value());
            } else {
                return Err(meta.error("unsupported #[py(...)] option"));
            }
            Ok(())
        })?;
    }
    Ok(out)
}

fn python_ident(name: &str, span: proc_macro2::Span) -> syn::Result<()> {
    let valid = !name.is_empty()
        && (name.chars().next().unwrap().is_ascii_alphabetic() || name.starts_with('_'));
    let valid = valid && name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_');
    let keywords = [
        "False", "None", "True", "and", "as", "assert", "async", "await", "break", "class",
        "continue", "def", "del", "elif", "else", "except", "finally", "for", "from", "global",
        "if", "import", "in", "is", "lambda", "nonlocal", "not", "or", "pass", "raise", "return",
        "try", "while", "with", "yield",
    ];
    if valid && !keywords.contains(&name) {
        Ok(())
    } else {
        Err(syn::Error::new(
            span,
            format!("{name:?} is not a valid Python identifier"),
        ))
    }
}

fn python_class_ident(name: &str, span: proc_macro2::Span) -> syn::Result<()> {
    python_ident(name, span)?;
    if matches!(
        name,
        "dataclass"
            | "field"
            | "Enum"
            | "Literal"
            | "TypeAlias"
            | "int"
            | "str"
            | "bool"
            | "float"
            | "list"
            | "dict"
            | "set"
            | "tuple"
    ) {
        return Err(syn::Error::new(
            span,
            format!("{name:?} conflicts with a Python name used by generated code"),
        ));
    }
    Ok(())
}

fn python_literal(value: &str) -> String {
    let mut out = String::from("\"");
    for ch in value.chars() {
        match ch {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if c.is_control() => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

fn fields(fields: &Fields) -> syn::Result<(Vec<Tokens>, Vec<Tokens>, bool)> {
    let mut lines = Vec::new();
    let mut deps = Vec::new();
    let mut names = std::collections::HashSet::new();
    for (index, field) in fields.iter().enumerate() {
        let attr = options(&field.attrs)?;
        if attr.export || attr.export_to.is_some() {
            return Err(syn::Error::new_spanned(
                field,
                "export options belong on the struct or enum",
            ));
        }
        if attr.skip {
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
        let ty = &field.ty;
        lines.push(quote! {
            out.push_str("    ");
            out.push_str(#name);
            out.push_str(": ");
            out.push_str(&<#ty as ::py_rs::PY>::inline());
            out.push('\n');
        });
        deps.push(quote! { deps.extend(<#ty as ::py_rs::PY>::referenced_types()); });
    }
    let empty = lines.is_empty();
    Ok((lines, deps, empty))
}

#[proc_macro_derive(PY, attributes(py))]
pub fn derive_py(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    match expand(input) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

fn expand(input: DeriveInput) -> syn::Result<Tokens> {
    if !input.generics.params.is_empty() {
        return Err(syn::Error::new_spanned(
            input.generics,
            "PY derive currently supports owned, non-generic types only",
        ));
    }
    let ident = input.ident;
    let attr = options(&input.attrs)?;
    if attr.skip {
        return Err(syn::Error::new(
            ident.span(),
            "#[py(skip)] belongs on a field or enum variant",
        ));
    }
    let name = attr.rename.unwrap_or_else(|| ident.to_string());
    python_class_ident(&name, ident.span())?;
    let output = attr.export_to.unwrap_or_else(|| format!("{name}.py"));
    let mut deps = Vec::new();
    let mut local_names = vec![name.clone()];
    let declaration = match input.data {
        Data::Struct(data) => {
            let (lines, field_deps, empty) = fields(&data.fields)?;
            deps.extend(field_deps);
            let pass = if empty {
                quote! { out.push_str("    pass\n"); }
            } else {
                quote! {}
            };
            quote! {
                let mut out = format!("from dataclasses import dataclass\n\n@dataclass\nclass {}:\n", #name);
                #(#lines)*
                #pass
                out
            }
        }
        Data::Enum(data) => {
            let mut variants = Vec::new();
            let all_unit = data
                .variants
                .iter()
                .filter(|v| options(&v.attrs).map(|o| !o.skip).unwrap_or(true))
                .all(|v| matches!(v.fields, Fields::Unit));
            for variant in &data.variants {
                let vattr = options(&variant.attrs)?;
                if vattr.export || vattr.export_to.is_some() {
                    return Err(syn::Error::new_spanned(
                        variant,
                        "export options belong on the enum",
                    ));
                }
                if vattr.skip {
                    continue;
                }
                let variant_ident = variant
                    .ident
                    .to_string()
                    .trim_start_matches("r#")
                    .to_owned();
                python_ident(&variant_ident, variant.ident.span())?;
                let value = vattr.rename.unwrap_or_else(|| variant_ident.clone());
                let value_literal = python_literal(&value);
                if all_unit {
                    variants.push(quote! { out.push_str(&format!("    {} = {}\n", #variant_ident, #value_literal)); });
                } else {
                    let class_name = format!("{}{}", name, variant_ident);
                    python_class_ident(&class_name, variant.ident.span())?;
                    local_names.push(class_name.clone());
                    let (lines, field_deps, _) = fields(&variant.fields)?;
                    if variant
                        .fields
                        .iter()
                        .filter_map(|field| {
                            let options = options(&field.attrs).ok()?;
                            if options.skip {
                                return None;
                            }
                            Some(options.rename.unwrap_or_else(|| {
                                field
                                    .ident
                                    .as_ref()
                                    .map(|i| i.to_string().trim_start_matches("r#").to_owned())
                                    .unwrap_or_default()
                            }))
                        })
                        .any(|field_name| field_name == "kind")
                    {
                        return Err(syn::Error::new_spanned(
                            &variant.fields,
                            "`kind` is reserved for the generated enum variant tag",
                        ));
                    }
                    deps.extend(field_deps);
                    variants.push(quote! {
                        out.push_str(&format!("@dataclass\nclass {}:\n    kind: Literal[{}] = field(default={}, init=False)\n", #class_name, #value_literal, #value_literal));
                        #(#lines)*
                        out.push('\n');
                    });
                }
            }
            if all_unit {
                let pass = if variants.is_empty() {
                    quote! { out.push_str("    pass\n"); }
                } else {
                    quote! {}
                };
                quote! {
                    let mut out = format!("from enum import Enum\n\nclass {}(str, Enum):\n", #name);
                    #(#variants)*
                    #pass
                    out
                }
            } else {
                let names: Vec<String> = data
                    .variants
                    .iter()
                    .filter_map(|v| match options(&v.attrs) {
                        Ok(o) if !o.skip => Some(format!("{}{}", name, v.ident)),
                        _ => None,
                    })
                    .collect();
                let alias = names.join(" | ");
                quote! {
                    let mut out = String::from("from dataclasses import dataclass, field\nfrom typing import Literal, TypeAlias\n\n");
                    #(#variants)*
                    out.push_str(&format!("{}: TypeAlias = {}\n", #name, #alias));
                    out
                }
            }
        }
        Data::Union(data) => {
            return Err(syn::Error::new_spanned(
                data.union_token,
                "PY cannot be derived for unions",
            ))
        }
    };
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
    Ok(quote! {
        impl ::py_rs::PY for #ident {
            fn name() -> String { #name.into() }
            fn inline() -> String { #name.into() }
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
