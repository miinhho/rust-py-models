use syn::{Attribute, LitBool, LitStr, Token};

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

    pub(crate) fn decorator(self) -> String {
        let mut args = Vec::new();
        for (name, value) in [
            ("frozen", self.frozen),
            ("slots", self.slots),
            ("kw_only", self.kw_only),
        ] {
            if let Some(value) = value {
                args.push(format!("{name}={}", if value { "True" } else { "False" }));
            }
        }
        if args.is_empty() {
            "@dataclass".into()
        } else {
            format!("@dataclass({})", args.join(", "))
        }
    }
}

#[derive(Default)]
pub(crate) struct Options {
    pub(crate) rename: Option<String>,
    pub(crate) skip: bool,
    pub(crate) export: bool,
    pub(crate) export_to: Option<String>,
    pub(crate) dataclass: DataclassOptions,
    pub(crate) python_type: Option<String>,
    pub(crate) import: Option<String>,
}

fn bool_option(meta: &syn::meta::ParseNestedMeta<'_>) -> syn::Result<bool> {
    if meta.input.peek(Token![=]) {
        Ok(meta.value()?.parse::<LitBool>()?.value())
    } else {
        Ok(true)
    }
}

pub(crate) fn options(attrs: &[Attribute]) -> syn::Result<Options> {
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
            } else if meta.path.is_ident("frozen") {
                out.dataclass.frozen = Some(bool_option(&meta)?);
            } else if meta.path.is_ident("slots") {
                out.dataclass.slots = Some(bool_option(&meta)?);
            } else if meta.path.is_ident("kw_only") {
                out.dataclass.kw_only = Some(bool_option(&meta)?);
            } else if meta.path.is_ident("type") {
                out.python_type = Some(meta.value()?.parse::<LitStr>()?.value());
            } else if meta.path.is_ident("import") {
                out.import = Some(meta.value()?.parse::<LitStr>()?.value());
            } else {
                return Err(meta.error("unsupported #[py(...)] option"));
            }
            Ok(())
        })?;
    }
    Ok(out)
}
