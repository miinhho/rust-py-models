use super::path;
use crate::{Dependency, ExportError, PY};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

/// Validate every concrete instantiation, then write one declaration per Rust type.
pub(super) fn collect<T: PY>(dir: &Path) -> Result<Vec<Dependency>, ExportError> {
    let root = T::dependency().ok_or(ExportError::NotExportable(std::any::type_name::<T>()))?;
    let mut seen_concrete = HashSet::<&'static str>::new();
    let mut seen_declarations = HashSet::<&'static str>::new();
    let mut paths = HashMap::<PathBuf, &'static str>::new();
    let mut planned = Vec::new();
    let mut pending = vec![root];

    while let Some(dep) = pending.pop() {
        if !seen_concrete.insert(dep.concrete_id) {
            continue;
        }
        path::validate(&dep.path)?;
        if paths
            .insert(dep.path.clone(), dep.id)
            .is_some_and(|id| id != dep.id)
        {
            return Err(ExportError::ConflictingFile(dir.join(dep.path)));
        }
        (dep.render)()?;
        pending.extend((dep.children)());
        if seen_declarations.insert(dep.id) {
            planned.push(dep);
        }
    }
    Ok(planned)
}
