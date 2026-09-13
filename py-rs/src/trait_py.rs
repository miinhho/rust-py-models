use crate::dependency::Dependency;
use crate::error::ExportError;
use crate::export;
use std::path::PathBuf;

/// Describes a Python representation of a Rust type.
///
/// Container implementations contribute referenced types; derived structs
/// and enums contribute their direct field dependencies.
pub trait PY {
    fn name() -> String;
    fn inline() -> String;

    /// Render this type with symbolic Python type arguments in a generic declaration.
    fn inline_with(_args: &[String]) -> String {
        Self::inline()
    }

    /// Stable identity of the generated Python declaration.
    fn declaration_id() -> &'static str {
        std::any::type_name::<Self>()
    }

    /// Python imports needed by this type's own declaration.
    fn prelude() -> String {
        String::new()
    }

    /// Standard-library or third-party imports needed by this type's annotation.
    fn imports() -> Vec<String> {
        Vec::new()
    }

    /// Helper aliases required by this declaration's field annotations.
    fn definitions() -> Vec<String> {
        Vec::new()
    }

    /// Imports required when this type appears inside another annotation.
    fn annotation_imports() -> Vec<String> {
        Vec::new()
    }

    /// Imports belonging to this type constructor, excluding its type arguments.
    fn local_imports() -> Vec<String> {
        Self::annotation_imports()
    }

    fn annotation_definitions() -> Vec<String> {
        Vec::new()
    }

    /// Whether the generated Python value can be hashed.
    fn is_hashable() -> bool {
        false
    }

    /// Check constraints that are needed for a usable Python annotation.
    fn validate() -> Result<(), ExportError> {
        Ok(())
    }

    /// Validate an annotation without traversing a referenced model's fields.
    fn annotation_validate() -> Result<(), ExportError> {
        Ok(())
    }

    fn decl() -> String {
        String::new()
    }

    fn dependencies() -> Vec<Dependency> {
        Vec::new()
    }

    fn output_path() -> Option<PathBuf> {
        None
    }

    fn declaration_names() -> Vec<String> {
        vec![Self::name()]
    }

    /// Types referred to by this type when it appears in a field.
    fn referenced_types() -> Vec<Dependency> {
        Self::dependency().into_iter().collect()
    }

    fn dependency() -> Option<Dependency> {
        Self::output_path().map(Dependency::of::<Self>)
    }

    /// Render one Python module, including its imports.
    fn export_to_string() -> Result<String, ExportError>
    where
        Self: Sized,
    {
        export::render::<Self>()
    }

    /// Write this declaration to `bindings/<name>.py` (or `#[py(export_to)]`).
    fn export() -> Result<(), ExportError>
    where
        Self: Sized,
    {
        export::export_one::<Self>(&export::export_dir())
    }

    /// Export this type and all of its transitive dependencies.
    fn export_all() -> Result<(), ExportError>
    where
        Self: Sized,
    {
        export::export_all::<Self>()
    }
}
