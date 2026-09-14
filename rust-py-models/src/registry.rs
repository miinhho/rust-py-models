use crate::{export, ExportError};
use std::path::Path;

/// A model graph registered by `#[py(export)]`.
#[doc(hidden)]
pub struct ExportRoot {
    name: &'static str,
    write: fn(&Path) -> Result<(), ExportError>,
}

impl ExportRoot {
    /// Creates an entry for a derived model.
    #[doc(hidden)]
    pub const fn new(name: &'static str, write: fn(&Path) -> Result<(), ExportError>) -> Self {
        Self { name, write }
    }
}

inventory::collect!(ExportRoot);

/// Writes all `#[py(export)]` model graphs linked into this executable.
///
/// Uses `bindings/` unless `RUST_PY_MODELS_EXPORT_DIR` is set. Deriving `PY`
/// and running tests do not write files on their own.
///
/// # Errors
///
/// Returns an error when there are no registered roots or an export fails.
pub fn export_all() -> Result<(), ExportError> {
    export_all_to(export::export_dir())
}

/// Writes all `#[py(export)]` model graphs below `dir`.
///
/// Registered roots are visited in Rust type-name order for deterministic output.
/// `dir` overrides `RUST_PY_MODELS_EXPORT_DIR`.
///
/// # Errors
///
/// Returns an error when there are no registered roots or an export fails.
pub fn export_all_to(dir: impl AsRef<Path>) -> Result<(), ExportError> {
    let mut roots = inventory::iter::<ExportRoot>
        .into_iter()
        .collect::<Vec<_>>();
    if roots.is_empty() {
        return Err(ExportError::NoExportRoots);
    }
    roots.sort_by_key(|root| root.name);
    for root in roots {
        (root.write)(dir.as_ref())?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_when_no_roots_are_registered() {
        let nonce = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("clock before Unix epoch")
            .as_nanos();
        let dir = std::env::temp_dir().join(format!(
            "rust-py-models-empty-registry-{}-{nonce}",
            std::process::id()
        ));
        assert!(matches!(
            export_all_to(&dir),
            Err(ExportError::NoExportRoots)
        ));
        assert!(!dir.exists());
    }
}
