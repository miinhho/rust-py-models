use super::path;
use crate::{ExportError, ModelSpec, PY};
use std::collections::{BTreeMap, HashSet};
use std::path::{Path, PathBuf};

pub(super) struct ExportPlan {
    modules: BTreeMap<PathBuf, Vec<ModelSpec>>,
}

impl ExportPlan {
    pub(super) fn remove(&mut self, path: &Path) -> Option<Vec<ModelSpec>> {
        self.modules.remove(path)
    }
}

impl IntoIterator for ExportPlan {
    type Item = (PathBuf, Vec<ModelSpec>);
    type IntoIter = std::collections::btree_map::IntoIter<PathBuf, Vec<ModelSpec>>;

    fn into_iter(self) -> Self::IntoIter {
        self.modules.into_iter()
    }
}

pub(super) fn collect<T: PY + ?Sized>() -> Result<ExportPlan, ExportError> {
    let root = T::model_spec()?.ok_or(ExportError::NotExportable(std::any::type_name::<T>()))?;
    let mut seen_concrete = HashSet::<&'static str>::new();
    let mut seen_declarations = HashSet::<&'static str>::new();
    let mut modules = BTreeMap::<PathBuf, Vec<ModelSpec>>::new();
    let mut portable_paths = BTreeMap::<String, PathBuf>::new();
    let mut pending = vec![root];

    while let Some(model) = pending.pop() {
        if !seen_concrete.insert(model.concrete_id()) {
            continue;
        }
        path::validate(model.output_path())?;
        let portable = model
            .output_path()
            .to_string_lossy()
            .replace('\\', "/")
            .to_ascii_lowercase();
        if let Some(existing) = portable_paths.get(&portable) {
            if existing != model.output_path() {
                return Err(ExportError::PortablePathCollision {
                    first: existing.clone(),
                    second: model.output_path().to_path_buf(),
                });
            }
        } else {
            portable_paths.insert(portable, model.output_path().to_path_buf());
        }
        model.validate()?;
        for dependency in model.dependencies() {
            pending.push(dependency.materialize()?);
        }
        if seen_declarations.insert(model.id()) {
            modules
                .entry(model.output_path().to_path_buf())
                .or_default()
                .push(model);
        }
    }
    Ok(ExportPlan { modules })
}
