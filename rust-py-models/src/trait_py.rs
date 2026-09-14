use crate::error::ExportError;
use crate::export;
use crate::{ModelSpec, TypeSpec};
use std::path::Path;

/// Describes the typed Python representation of a Rust type.
pub trait PY {
    /// Python annotation and all requirements carried by that annotation.
    fn type_spec() -> TypeSpec;

    /// Build this type constructor with symbolic generic arguments.
    fn type_spec_with(_args: &[TypeSpec]) -> TypeSpec {
        Self::type_spec()
    }

    /// Complete declaration emitted when this Rust type is exported.
    ///
    /// # Errors
    ///
    /// Returns an [`ExportError`] when declaration construction fails.
    fn model_spec() -> Result<Option<ModelSpec>, ExportError> {
        Ok(None)
    }

    /// Render only this type's Python annotation.
    #[must_use]
    fn inline() -> String {
        Self::type_spec().annotation()
    }

    /// Whether this type's Python representation can be hashed.
    #[must_use]
    fn is_hashable() -> bool {
        Self::type_spec().is_hashable()
    }

    /// Render one Python module, including its imports.
    ///
    /// # Errors
    ///
    /// Returns an [`ExportError`] when the declaration or its dependencies are invalid.
    fn export_to_string() -> Result<String, ExportError>
    where
        Self: Sized,
    {
        export::render::<Self>()
    }

    /// Write this declaration to `bindings/<name>.py` (or `#[py(export_to)]`).
    ///
    /// # Errors
    ///
    /// Returns an [`ExportError`] when rendering fails or the destination cannot be written.
    fn export() -> Result<(), ExportError>
    where
        Self: Sized,
    {
        export::export_one::<Self>(&export::export_dir())
    }

    /// Export this type and all transitive dependencies to the configured directory.
    ///
    /// # Errors
    ///
    /// Returns an [`ExportError`] when any declaration is invalid or cannot be written.
    fn export_all() -> Result<(), ExportError>
    where
        Self: Sized,
    {
        export::export_all::<Self>()
    }

    /// Export this type and all transitive dependencies to an explicit directory.
    ///
    /// This ignores `RUST_PY_MODELS_EXPORT_DIR`; `#[py(export_to = "...")]`
    /// paths remain relative to `dir`.
    ///
    /// # Errors
    ///
    /// Returns an [`ExportError`] when any declaration is invalid or cannot be written.
    fn export_all_to(dir: impl AsRef<Path>) -> Result<(), ExportError>
    where
        Self: Sized,
    {
        export::export_all_to::<Self>(dir.as_ref())
    }
}
