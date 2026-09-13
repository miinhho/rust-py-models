use std::fmt;
use std::path::PathBuf;

/// Errors produced while generating Python files.
#[derive(Debug)]
pub enum ExportError {
    NotExportable(&'static str),
    InvalidPath(PathBuf),
    ConflictingFile(PathBuf),
    NameCollision(String),
    UnhashableType(String),
    Io(std::io::Error),
}

impl fmt::Display for ExportError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotExportable(name) => write!(f, "{name} has no Python declaration"),
            Self::InvalidPath(path) => write!(f, "invalid export path: {}", path.display()),
            Self::ConflictingFile(path) => write!(
                f,
                "refusing to overwrite a file owned by another type: {}",
                path.display()
            ),
            Self::NameCollision(name) => write!(f, "conflicting Python type name: {name}"),
            Self::UnhashableType(name) => {
                write!(
                    f,
                    "{name} cannot be used as a Python set element or dict key"
                )
            }
            Self::Io(err) => err.fmt(f),
        }
    }
}

impl std::error::Error for ExportError {}

impl From<std::io::Error> for ExportError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}
