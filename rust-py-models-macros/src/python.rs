use proc_macro2::Span;

pub(crate) fn python_ident(name: &str, span: Span) -> syn::Result<()> {
    let valid = matches!(name.chars().next(), Some(c) if c.is_ascii_alphabetic() || c == '_');
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

pub(crate) fn python_class_ident(name: &str, span: Span) -> syn::Result<()> {
    python_ident(name, span)?;
    if matches!(
        name,
        "dataclass"
            | "field"
            | "Enum"
            | "Literal"
            | "TypeAlias"
            | "Generic"
            | "TypeVar"
            | "_PyRsJsonValue"
            | "collections"
            | "datetime"
            | "ipaddress"
            | "pathlib"
            | "uuid"
            | "int"
            | "str"
            | "bool"
            | "float"
            | "bytes"
            | "object"
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
