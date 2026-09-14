use crate::{ExportError, ModelSpec, PY};
use std::path::{Path, PathBuf};

/// An exported Rust type referred to by another type.
#[doc(hidden)]
#[derive(Clone)]
pub struct Dependency {
    id: &'static str,
    concrete_id: &'static str,
    name: String,
    path: PathBuf,
    model: fn() -> Result<Option<ModelSpec>, ExportError>,
}

impl PartialEq for Dependency {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
            && self.concrete_id == other.concrete_id
            && self.name == other.name
            && self.path == other.path
    }
}

impl Eq for Dependency {}

impl Dependency {
    #[must_use]
    pub fn of<T: PY + ?Sized>(
        id: &'static str,
        name: impl Into<String>,
        path: impl Into<PathBuf>,
    ) -> Self {
        Self {
            id,
            concrete_id: std::any::type_name::<T>(),
            name: name.into(),
            path: path.into(),
            model: T::model_spec,
        }
    }

    /// Primary Python name imported for this dependency.
    #[must_use]
    pub fn python_name(&self) -> &str {
        &self.name
    }

    /// Output path relative to the bindings directory.
    #[must_use]
    pub fn output_path(&self) -> &Path {
        &self.path
    }

    pub(crate) fn id(&self) -> &'static str {
        self.id
    }

    pub(crate) fn materialize(&self) -> Result<ModelSpec, ExportError> {
        (self.model)()?.ok_or(ExportError::NotExportable(self.concrete_id))
    }
}
