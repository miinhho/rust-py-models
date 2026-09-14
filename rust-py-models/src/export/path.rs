use crate::ExportError;
use std::path::{Component, Path};

pub(super) fn validate(path: &Path) -> Result<(), ExportError> {
    if path.extension().and_then(|x| x.to_str()) != Some("py")
        || path
            .components()
            .any(|c| !matches!(c, Component::Normal(_)))
    {
        return Err(ExportError::InvalidPath(path.to_path_buf()));
    }
    let mut parts = path
        .components()
        .map(|c| c.as_os_str().to_string_lossy().into_owned())
        .collect::<Vec<_>>();
    let module = parts.pop().unwrap_or_default();
    let module = module.strip_suffix(".py").unwrap_or("");
    if module == "__init__"
        || !valid_module_name(module)
        || parts.iter().any(|part| !valid_module_name(part))
    {
        return Err(ExportError::InvalidPath(path.to_path_buf()));
    }
    Ok(())
}

fn valid_module_name(name: &str) -> bool {
    let mut chars = name.chars();
    matches!(chars.next(), Some(c) if c.is_ascii_alphabetic() || c == '_')
        && chars.all(|c| c.is_ascii_alphanumeric() || c == '_')
        && !matches!(
            name,
            "False"
                | "None"
                | "True"
                | "and"
                | "as"
                | "assert"
                | "async"
                | "await"
                | "break"
                | "class"
                | "continue"
                | "def"
                | "del"
                | "elif"
                | "else"
                | "except"
                | "finally"
                | "for"
                | "from"
                | "global"
                | "if"
                | "import"
                | "in"
                | "is"
                | "lambda"
                | "nonlocal"
                | "not"
                | "or"
                | "pass"
                | "raise"
                | "return"
                | "try"
                | "while"
                | "with"
                | "yield"
        )
}

pub(super) fn relative_module(from: &Path, to: &Path) -> String {
    let from_dir = from.parent().unwrap_or(Path::new(""));
    let to_dir = to.parent().unwrap_or(Path::new(""));
    let from_parts: Vec<_> = from_dir.components().collect();
    let to_parts: Vec<_> = to_dir.components().collect();
    let common = from_parts
        .iter()
        .zip(&to_parts)
        .take_while(|(a, b)| a == b)
        .count();
    let mut module = ".".repeat(from_parts.len() - common + 1);
    let tail: Vec<_> = to_parts[common..]
        .iter()
        .map(|c| c.as_os_str().to_string_lossy().into_owned())
        .collect();
    if !tail.is_empty() {
        module.push_str(&tail.join("."));
        module.push('.');
    }
    module.push_str(
        &to.file_stem()
            .expect("validated Python module path has a file stem")
            .to_string_lossy(),
    );
    module
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn relative_paths_cover_parent_and_sibling_packages() {
        assert_eq!(
            relative_module(Path::new("A.py"), Path::new("models/B.py")),
            ".models.B"
        );
        assert_eq!(
            relative_module(Path::new("models/B.py"), Path::new("A.py")),
            "..A"
        );
        assert_eq!(
            relative_module(Path::new("models/B.py"), Path::new("models/C.py")),
            ".C"
        );
    }

    #[test]
    fn rejects_invalid_python_module_paths() {
        for bad in ["../A.py", "class/A.py", "bad-name/A.py", "__init__.py"] {
            assert!(matches!(
                validate(Path::new(bad)),
                Err(ExportError::InvalidPath(_))
            ));
        }
    }
}
