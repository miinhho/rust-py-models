mod imports;
mod path;
mod plan;
mod render;
mod write;

use crate::{ExportError, PY};
use std::path::{Path, PathBuf};

pub(crate) use render::render;

pub(crate) fn export_dir() -> PathBuf {
    std::env::var_os("PY_RS_EXPORT_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("bindings"))
}

pub(crate) fn export_one<T: PY + ?Sized>(dir: &Path) -> Result<(), ExportError> {
    let relative =
        T::output_path().ok_or(ExportError::NotExportable(std::any::type_name::<T>()))?;
    path::validate(&relative)?;
    write::module::<T>(dir, &relative, &render::<T>()?)
}

pub(crate) fn export_all<T: PY>() -> Result<(), ExportError> {
    let dir = export_dir();
    for dep in plan::collect::<T>(&dir)? {
        (dep.export)(&dir)?;
    }
    Ok(())
}
