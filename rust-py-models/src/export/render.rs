use super::imports::ImportPlan;
use super::plan;
use crate::ir::render_module;
use crate::{Declaration, ExportError, ModelSpec, PY};
use std::collections::{BTreeMap, BTreeSet, HashSet};
use std::path::PathBuf;

pub(crate) fn render<T: PY + ?Sized>() -> Result<String, ExportError> {
    let root = T::model_spec()?.ok_or(ExportError::NotExportable(std::any::type_name::<T>()))?;
    let own = root.output_path().to_path_buf();
    let mut plan = plan::collect::<T>()?;
    let owners = plan
        .remove(&own)
        .ok_or(ExportError::NotExportable(std::any::type_name::<T>()))?;
    models(own, owners)
}

pub(super) fn models(own: PathBuf, mut models: Vec<ModelSpec>) -> Result<String, ExportError> {
    models.sort_by_key(ModelSpec::id);
    let imports = ImportPlan::for_models(own.clone(), &models)?;
    let mut module_imports = BTreeSet::new();
    for model in &models {
        module_imports.extend(model.imports());
    }

    let mut typevars = BTreeSet::new();
    let mut definitions = BTreeMap::<String, Declaration>::new();
    let mut declaration_names = HashSet::new();
    let mut declarations = Vec::new();
    for model in &models {
        for definition in model.definitions()? {
            let name = definition.name().to_owned();
            if declaration_names.contains(&name)
                || definitions
                    .get(&name)
                    .is_some_and(|existing| existing != &definition)
            {
                return Err(ExportError::NameCollision(name));
            }
            definitions.entry(name).or_insert(definition);
        }
        for declaration in model.declarations() {
            match declaration {
                Declaration::TypeVar(name) => {
                    typevars.insert(name.clone());
                }
                declaration => {
                    let name = declaration.name().to_owned();
                    if !declaration_names.insert(name.clone()) || definitions.contains_key(&name) {
                        return Err(ExportError::NameCollision(name));
                    }
                    declarations.push(declaration.clone());
                }
            }
        }
    }

    let declarations = typevars
        .into_iter()
        .map(Declaration::TypeVar)
        .chain(definitions.into_values())
        .chain(declarations)
        .collect::<Vec<_>>();
    let owners = models.iter().map(ModelSpec::id).collect::<Vec<_>>();
    let model_imports = imports.entries();
    Ok(render_module(
        &owners,
        &module_imports,
        &model_imports,
        &declarations,
    ))
}
