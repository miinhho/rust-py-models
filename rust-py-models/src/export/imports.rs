use super::path;
use crate::ir::ModelImport;
use crate::{Declaration, Dependency, ExportError, ModelSpec};
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::path::PathBuf;

pub(super) struct ImportPlan {
    own_ids: HashSet<&'static str>,
    own_path: PathBuf,
    names: HashMap<String, &'static str>,
    eager: BTreeMap<String, BTreeSet<String>>,
    cyclic: BTreeMap<String, BTreeSet<String>>,
}

impl ImportPlan {
    pub(super) fn for_models(own_path: PathBuf, models: &[ModelSpec]) -> Result<Self, ExportError> {
        let own_ids = models.iter().map(ModelSpec::id).collect::<HashSet<_>>();
        let mut plan = Self {
            own_ids,
            own_path,
            names: HashMap::new(),
            eager: BTreeMap::new(),
            cyclic: BTreeMap::new(),
        };
        let mut typevars = HashSet::new();
        for model in models {
            for definition in model.definitions()? {
                plan.add_local_name(definition.name(), model.id(), false, &mut typevars)?;
            }
            for declaration in model.declarations() {
                plan.add_local_name(
                    declaration.name(),
                    model.id(),
                    matches!(declaration, Declaration::TypeVar(_)),
                    &mut typevars,
                )?;
            }
        }
        for model in models {
            for dependency in model.dependencies() {
                plan.add(&dependency)?;
            }
        }
        Ok(plan)
    }

    pub(super) fn entries(&self) -> Vec<ModelImport> {
        let mut entries = self
            .eager
            .iter()
            .map(|(module, names)| ModelImport {
                module: module.clone(),
                names: names.clone(),
                cyclic: false,
            })
            .chain(self.cyclic.iter().map(|(module, names)| ModelImport {
                module: module.clone(),
                names: names.clone(),
                cyclic: true,
            }))
            .collect::<Vec<_>>();
        entries.sort_by(|left, right| {
            left.cyclic.cmp(&right.cyclic).then_with(|| {
                left.module
                    .to_ascii_lowercase()
                    .cmp(&right.module.to_ascii_lowercase())
                    .then_with(|| left.module.cmp(&right.module))
            })
        });
        entries
    }

    fn add_local_name(
        &mut self,
        name: &str,
        id: &'static str,
        typevar: bool,
        typevars: &mut HashSet<String>,
    ) -> Result<(), ExportError> {
        if let Some(existing) = self.names.get(name) {
            if *existing != id && !(typevar && typevars.contains(name)) {
                return Err(ExportError::NameCollision(name.to_owned()));
            }
        } else {
            self.names.insert(name.to_owned(), id);
        }
        if typevar {
            typevars.insert(name.to_owned());
        }
        Ok(())
    }

    fn add(&mut self, dependency: &Dependency) -> Result<(), ExportError> {
        if self.own_ids.contains(dependency.id()) || self.own_path == dependency.output_path() {
            return Ok(());
        }
        path::validate(dependency.output_path())?;
        let id = dependency.id();
        let name = dependency.python_name().to_owned();
        if self
            .names
            .insert(name.clone(), id)
            .is_some_and(|existing| existing != id)
        {
            return Err(ExportError::NameCollision(name));
        }
        let module = path::relative_module(&self.own_path, dependency.output_path());
        let imports = if reaches_any_type(dependency, &self.own_ids, &mut HashSet::new())? {
            &mut self.cyclic
        } else {
            &mut self.eager
        };
        imports.entry(module).or_default().insert(name);
        Ok(())
    }
}

fn reaches_any_type(
    dependency: &Dependency,
    targets: &HashSet<&'static str>,
    seen: &mut HashSet<&'static str>,
) -> Result<bool, ExportError> {
    if targets.contains(dependency.id()) {
        return Ok(true);
    }
    if !seen.insert(dependency.id()) {
        return Ok(false);
    }
    let model = dependency.materialize()?;
    for child in model.dependencies() {
        if reaches_any_type(&child, targets, seen)? {
            return Ok(true);
        }
    }
    Ok(false)
}
