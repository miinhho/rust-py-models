use crate::error::ExportError;
use crate::export;
use crate::PY;
use std::path::{Path, PathBuf};

/// An exported Rust type referred to by another type.
#[derive(Clone)]
pub struct Dependency {
    pub(crate) id: &'static str,
    pub(crate) concrete_id: &'static str,
    pub(crate) name: String,
    pub(crate) path: PathBuf,
    pub(crate) render: fn() -> Result<String, ExportError>,
    pub(crate) export: fn(&Path) -> Result<(), ExportError>,
    pub(crate) children: fn() -> Vec<Dependency>,
}

impl Dependency {
    pub(crate) fn of<T: PY + ?Sized>(path: PathBuf) -> Self {
        Self {
            id: T::declaration_id(),
            concrete_id: std::any::type_name::<T>(),
            name: T::name(),
            path,
            render: export::render::<T>,
            export: export::export_one::<T>,
            children: T::dependencies,
        }
    }
}
