mod imports;
mod path;
mod plan;
mod render;
mod write;

use crate::{ExportError, PY};
use std::path::{Path, PathBuf};

pub(crate) use render::render;

pub(crate) fn export_dir() -> PathBuf {
    std::env::var_os("RUST_PY_MODELS_EXPORT_DIR")
        .map_or_else(|| PathBuf::from("bindings"), PathBuf::from)
}

pub(crate) fn export_one<T: PY + ?Sized>(dir: &Path) -> Result<(), ExportError> {
    let model = T::model_spec()?.ok_or(ExportError::NotExportable(std::any::type_name::<T>()))?;
    path::validate(model.output_path())?;
    let relative = model.output_path().to_path_buf();
    let content = render::models(relative.clone(), vec![model])?;
    write::modules(dir, std::any::type_name::<T>(), vec![(relative, content)])
}

pub(crate) fn export_all<T: PY>() -> Result<(), ExportError> {
    export_all_to::<T>(&export_dir())
}

pub(crate) fn export_all_to<T: PY>(dir: &Path) -> Result<(), ExportError> {
    let mut outputs = Vec::new();
    for (relative, models) in plan::collect::<T>()? {
        let content = render::models(relative.clone(), models)?;
        outputs.push((relative, content));
    }
    write::modules(dir, std::any::type_name::<T>(), outputs)
}
