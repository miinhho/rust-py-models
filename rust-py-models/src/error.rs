use std::path::PathBuf;

/// Errors produced while generating Python files.
#[derive(Debug, thiserror::Error)]
pub enum ExportError {
    /// No marked export roots are linked into the current executable.
    #[error("no #[py(export)] roots are registered in this executable")]
    NoExportRoots,
    /// The requested Rust type does not define a Python declaration.
    #[error("{0} has no Python declaration")]
    NotExportable(&'static str),
    /// Serde flattening was requested for a type that is not a dataclass.
    #[error("{0} cannot be flattened into a Python dataclass")]
    NotFlattenable(&'static str),
    /// An export path is not a relative Python module path.
    #[error("invalid export path: {}", .0.display())]
    InvalidPath(PathBuf),
    /// Writing would replace a file not owned by the current export.
    #[error("refusing to overwrite a file owned by another type: {}", .0.display())]
    ConflictingFile(PathBuf),
    /// Two output paths would collide on a case-insensitive filesystem.
    #[error(
        "Python output paths differ only by case: {} and {}",
        first.display(),
        second.display()
    )]
    PortablePathCollision {
        /// First path participating in the collision.
        first: PathBuf,
        /// Second path participating in the collision.
        second: PathBuf,
    },
    /// The ownership manifest cannot be parsed or contains an invalid path.
    #[error("invalid rust-py-models manifest: {}", .0.display())]
    InvalidManifest(PathBuf),
    /// Multiple declarations or definitions introduce the same Python name.
    #[error("conflicting Python type name: {0}")]
    NameCollision(String),
    /// A configured name is not a valid supported Python identifier.
    #[error("invalid Python identifier: {0:?}")]
    InvalidPythonIdentifier(String),
    /// A generated type alias has no variants.
    #[error("Python type alias {0:?} has no variants")]
    EmptyTypeAlias(String),
    /// A union was constructed without any member types.
    #[error("Python union has no variants")]
    EmptyUnion,
    /// A set element or dictionary key maps to an unhashable Python type.
    #[error("{0} cannot be used as a Python set element or dict key")]
    UnhashableType(String),
    /// The filesystem operation failed.
    #[error(transparent)]
    Io(#[from] std::io::Error),
}
