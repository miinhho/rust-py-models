use std::path::PathBuf;

/// Errors produced while generating Python files.
#[derive(Debug, thiserror::Error)]
pub enum ExportError {
    #[error("{0} has no Python declaration")]
    NotExportable(&'static str),
    #[error("{0} cannot be flattened into a Python dataclass")]
    NotFlattenable(&'static str),
    #[error("invalid export path: {}", .0.display())]
    InvalidPath(PathBuf),
    #[error("refusing to overwrite a file owned by another type: {}", .0.display())]
    ConflictingFile(PathBuf),
    #[error(
        "Python output paths differ only by case: {} and {}",
        first.display(),
        second.display()
    )]
    PortablePathCollision { first: PathBuf, second: PathBuf },
    #[error("invalid rust-py-models manifest: {}", .0.display())]
    InvalidManifest(PathBuf),
    #[error("conflicting Python type name: {0}")]
    NameCollision(String),
    #[error("invalid Python identifier: {0:?}")]
    InvalidPythonIdentifier(String),
    #[error("Python type alias {0:?} has no variants")]
    EmptyTypeAlias(String),
    #[error("Python union has no variants")]
    EmptyUnion,
    #[error("{0} cannot be used as a Python set element or dict key")]
    UnhashableType(String),
    #[error(transparent)]
    Io(#[from] std::io::Error),
}
