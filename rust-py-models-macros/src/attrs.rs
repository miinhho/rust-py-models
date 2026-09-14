use heck::{
    ToKebabCase, ToLowerCamelCase, ToShoutyKebabCase, ToShoutySnakeCase, ToSnakeCase,
    ToUpperCamelCase,
};
use syn::parse::Parser;
#[cfg(feature = "serde-compat")]
use syn::punctuated::Punctuated;
use syn::{Attribute, Ident, LitBool, LitStr, Meta, Token, Type, WherePredicate};

#[derive(Clone, Copy, Default)]
pub(crate) struct DataclassOptions {
    pub(crate) frozen: Option<bool>,
    pub(crate) slots: Option<bool>,
    pub(crate) kw_only: Option<bool>,
}

impl DataclassOptions {
    pub(crate) fn is_set(self) -> bool {
        self.frozen.is_some() || self.slots.is_some() || self.kw_only.is_some()
    }

    pub(crate) fn with_overrides(self, overrides: Self) -> Self {
        Self {
            frozen: overrides.frozen.or(self.frozen),
            slots: overrides.slots.or(self.slots),
            kw_only: overrides.kw_only.or(self.kw_only),
        }
    }
}

#[cfg_attr(not(feature = "serde-compat"), allow(dead_code))]
#[derive(Clone, Copy)]
pub(crate) enum RenameRule {
    Lowercase,
    Uppercase,
    PascalCase,
    CamelCase,
    SnakeCase,
    ScreamingSnakeCase,
    KebabCase,
    ScreamingKebabCase,
}

impl RenameRule {
    #[cfg(feature = "serde-compat")]
    fn parse(value: &LitStr) -> syn::Result<Self> {
        match value.value().as_str() {
            "lowercase" => Ok(Self::Lowercase),
            "UPPERCASE" => Ok(Self::Uppercase),
            "PascalCase" => Ok(Self::PascalCase),
            "camelCase" => Ok(Self::CamelCase),
            "snake_case" => Ok(Self::SnakeCase),
            "SCREAMING_SNAKE_CASE" => Ok(Self::ScreamingSnakeCase),
            "kebab-case" => Ok(Self::KebabCase),
            "SCREAMING-KEBAB-CASE" => Ok(Self::ScreamingKebabCase),
            _ => Err(syn::Error::new_spanned(
                value,
                "unsupported serde rename rule",
            )),
        }
    }

    pub(crate) fn apply(self, value: &str) -> String {
        match self {
            Self::Lowercase => value.to_lowercase(),
            Self::Uppercase => value.to_uppercase(),
            Self::PascalCase => value.to_upper_camel_case(),
            Self::CamelCase => value.to_lower_camel_case(),
            Self::SnakeCase => value.to_snake_case(),
            Self::ScreamingSnakeCase => value.to_shouty_snake_case(),
            Self::KebabCase => value.to_kebab_case(),
            Self::ScreamingKebabCase => value.to_shouty_kebab_case(),
        }
    }
}

#[derive(Clone, Default)]
pub(crate) struct Options {
    pub(crate) rename: Option<String>,
    pub(crate) serde_rename: Option<String>,
    pub(crate) serde_rename_all: Option<RenameRule>,
    pub(crate) serde_rename_all_fields: Option<RenameRule>,
    pub(crate) serde_skip: bool,
    pub(crate) serde_flatten: bool,
    pub(crate) serde_tag: Option<String>,
    pub(crate) serde_content: Option<String>,
    pub(crate) serde_untagged: bool,
    pub(crate) skip: bool,
    pub(crate) export: bool,
    pub(crate) export_to: Option<String>,
    pub(crate) dataclass: DataclassOptions,
    pub(crate) newtype: bool,
    pub(crate) unsafe_python_type: Option<String>,
    pub(crate) import: Option<String>,
    pub(crate) as_type: Option<Type>,
    pub(crate) concrete: Vec<(Ident, Type)>,
    pub(crate) documentation: Option<String>,
    pub(crate) bound: Option<Vec<WherePredicate>>,
}

impl Options {
    pub(crate) fn effective_rename(&self) -> Option<&str> {
        self.rename.as_deref().or(self.serde_rename.as_deref())
    }

    pub(crate) fn is_skipped(&self) -> bool {
        self.skip || self.serde_skip
    }
}

fn bool_option(meta: &syn::meta::ParseNestedMeta<'_>) -> syn::Result<bool> {
    if meta.input.peek(Token![=]) {
        Ok(meta.value()?.parse::<LitBool>()?.value())
    } else {
        Ok(true)
    }
}

pub(crate) fn options(attrs: &[Attribute]) -> syn::Result<Options> {
    let mut out = Options {
        documentation: documentation(attrs)?,
        ..Options::default()
    };
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
            } else if meta.path.is_ident("frozen") {
                out.dataclass.frozen = Some(bool_option(&meta)?);
            } else if meta.path.is_ident("slots") {
                out.dataclass.slots = Some(bool_option(&meta)?);
            } else if meta.path.is_ident("kw_only") {
                out.dataclass.kw_only = Some(bool_option(&meta)?);
            } else if meta.path.is_ident("newtype") {
                out.newtype = true;
            } else if meta.path.is_ident("unsafe_type") {
                out.unsafe_python_type = Some(meta.value()?.parse::<LitStr>()?.value());
            } else if meta.path.is_ident("import") {
                out.import = Some(meta.value()?.parse::<LitStr>()?.value());
            } else if meta.path.is_ident("as") {
                let value = meta.value()?.parse::<LitStr>()?;
                out.as_type = Some(value.parse()?);
            } else if meta.path.is_ident("bound") {
                let value = meta.value()?.parse::<LitStr>()?;
                let predicates =
                    syn::punctuated::Punctuated::<WherePredicate, Token![,]>::parse_terminated
                        .parse_str(&value.value())?;
                out.bound = Some(predicates.into_iter().collect());
            } else if meta.path.is_ident("concrete") {
                meta.parse_nested_meta(|binding| {
                    let ident =
                        binding.path.get_ident().cloned().ok_or_else(|| {
                            binding.error("concrete parameter must be an identifier")
                        })?;
                    let ty = binding.value()?.parse::<Type>()?;
                    out.concrete.push((ident, ty));
                    Ok(())
                })?;
            } else {
                return Err(meta.error("unsupported #[py(...)] option"));
            }
            Ok(())
        })?;
    }
    #[cfg(feature = "serde-compat")]
    parse_serde_options(attrs, &mut out)?;
    Ok(out)
}

#[cfg(feature = "serde-compat")]
fn parse_serde_options(attrs: &[Attribute], out: &mut Options) -> syn::Result<()> {
    for attr in attrs.iter().filter(|a| a.path().is_ident("serde")) {
        let items = attr.parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)?;
        for item in items {
            match item {
                Meta::Path(path) if path.is_ident("skip") => out.serde_skip = true,
                Meta::Path(path) if path.is_ident("flatten") => out.serde_flatten = true,
                Meta::Path(path) if path.is_ident("default") => {}
                Meta::Path(path) if path.is_ident("untagged") => out.serde_untagged = true,
                Meta::NameValue(value) if value.path.is_ident("rename") => {
                    set_string(&mut out.serde_rename, &value.value, "rename")?;
                }
                Meta::NameValue(value) if value.path.is_ident("rename_all") => {
                    let rule = RenameRule::parse(&string_literal(&value.value, "rename_all")?)?;
                    if out.serde_rename_all.replace(rule).is_some() {
                        return Err(syn::Error::new_spanned(
                            value,
                            "duplicate serde rename_all attribute",
                        ));
                    }
                }
                Meta::NameValue(value) if value.path.is_ident("rename_all_fields") => {
                    let rule =
                        RenameRule::parse(&string_literal(&value.value, "rename_all_fields")?)?;
                    if out.serde_rename_all_fields.replace(rule).is_some() {
                        return Err(syn::Error::new_spanned(
                            value,
                            "duplicate serde rename_all_fields attribute",
                        ));
                    }
                }
                Meta::NameValue(value) if value.path.is_ident("tag") => {
                    set_string(&mut out.serde_tag, &value.value, "tag")?;
                }
                Meta::NameValue(value) if value.path.is_ident("content") => {
                    set_string(&mut out.serde_content, &value.value, "content")?;
                }
                Meta::NameValue(value) if value.path.is_ident("default") => {}
                _ => {}
            }
        }
    }
    Ok(())
}

#[cfg(feature = "serde-compat")]
fn set_string(slot: &mut Option<String>, value: &syn::Expr, name: &str) -> syn::Result<()> {
    let value = string_literal(value, name)?;
    if slot.replace(value.value()).is_some() {
        return Err(syn::Error::new_spanned(
            value,
            format!("duplicate serde {name} attribute"),
        ));
    }
    Ok(())
}

#[cfg(feature = "serde-compat")]
fn string_literal(value: &syn::Expr, name: &str) -> syn::Result<LitStr> {
    let syn::Expr::Lit(expression) = value else {
        return Err(syn::Error::new_spanned(
            value,
            format!("serde {name} must be a string"),
        ));
    };
    let syn::Lit::Str(value) = &expression.lit else {
        return Err(syn::Error::new_spanned(
            &expression.lit,
            format!("serde {name} must be a string"),
        ));
    };
    Ok(value.clone())
}

fn documentation(attrs: &[Attribute]) -> syn::Result<Option<String>> {
    let mut lines = Vec::new();
    for attr in attrs.iter().filter(|attr| attr.path().is_ident("doc")) {
        let Meta::NameValue(value) = &attr.meta else {
            continue;
        };
        let syn::Expr::Lit(expression) = &value.value else {
            continue;
        };
        let syn::Lit::Str(value) = &expression.lit else {
            continue;
        };
        let line = value.value();
        lines.push(line.strip_prefix(' ').unwrap_or(&line).to_owned());
    }

    for attr in attrs
        .iter()
        .filter(|attr| attr.path().is_ident("deprecated"))
    {
        let mut note = None;
        let mut since = None;
        if matches!(&attr.meta, Meta::List(_)) {
            attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("note") {
                    note = Some(meta.value()?.parse::<LitStr>()?.value());
                } else if meta.path.is_ident("since") {
                    since = Some(meta.value()?.parse::<LitStr>()?.value());
                }
                Ok(())
            })?;
        }
        let mut deprecated = String::from("Deprecated");
        if let Some(note) = note {
            deprecated.push_str(": ");
            deprecated.push_str(&note);
        }
        if let Some(since) = since {
            deprecated.push_str(" (since ");
            deprecated.push_str(&since);
            deprecated.push(')');
        }
        if !deprecated.ends_with('.') {
            deprecated.push('.');
        }
        if !lines.is_empty() {
            lines.push(String::new());
        }
        lines.push(deprecated);
    }

    if lines.is_empty() {
        Ok(None)
    } else {
        Ok(Some(lines.join("\n")))
    }
}
