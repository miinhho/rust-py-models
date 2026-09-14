use crate::error::ExportError;
use crate::export;
use crate::{ModelSpec, TypeSpec};
use std::path::Path;

/// Describes the Python annotation and optional declaration for a Rust type.
///
/// Deriving `PY` is the normal implementation path for structs and enums.
/// Manual implementations should build annotations with [`TypeSpec`] so imports
/// and model dependencies remain available to the exporter.
pub trait PY {
    /// Returns this Rust type's Python annotation and its requirements.
    fn type_spec() -> TypeSpec;

    /// Applies symbolic arguments when this Rust type is used as a generic constructor.
    fn type_spec_with(_args: &[TypeSpec]) -> TypeSpec {
        Self::type_spec()
    }

    /// Returns the declaration exported for this Rust type.
    ///
    /// Scalar and transparent mappings return `Ok(None)`.
    ///
    /// # Errors
    ///
    /// Returns an error if the declaration cannot be constructed.
    fn model_spec() -> Result<Option<ModelSpec>, ExportError> {
        Ok(None)
    }

    /// Renders this type's annotation without a declaration or imports.
    #[must_use]
    fn inline() -> String {
        Self::type_spec().annotation()
    }

    /// Returns whether the Python representation may be a set item or dict key.
    #[must_use]
    fn is_hashable() -> bool {
        Self::type_spec().is_hashable()
    }

    /// Renders the module for this type without writing it.
    ///
    /// # Errors
    ///
    /// Returns an error if this type or a referenced declaration is invalid.
    fn export_to_string() -> Result<String, ExportError>
    where
        Self: Sized,
    {
        export::render::<Self>()
    }

    /// Writes only this type's declaration to the configured export directory.
    ///
    /// # Errors
    ///
    /// Returns an error if rendering or writing fails.
    fn export() -> Result<(), ExportError>
    where
        Self: Sized,
    {
        export::export_one::<Self>(&export::export_dir())
    }

    /// Writes this type and all transitive model dependencies.
    ///
    /// # Errors
    ///
    /// Returns an error if any declaration is invalid or any file cannot be written.
    fn export_all() -> Result<(), ExportError>
    where
        Self: Sized,
    {
        export::export_all::<Self>()
    }

    /// Writes this type and all transitive dependencies below `dir`.
    ///
    /// `dir` replaces the configured export directory. Paths set by
    /// `#[py(export_to = "...")]` remain relative to it.
    ///
    /// # Errors
    ///
    /// Returns an error if any declaration is invalid or any file cannot be written.
    fn export_all_to(dir: impl AsRef<Path>) -> Result<(), ExportError>
    where
        Self: Sized,
    {
        export::export_all_to::<Self>(dir.as_ref())
    }
}
