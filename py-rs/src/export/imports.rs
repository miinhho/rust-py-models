use super::path;
use crate::{Dependency, ExportError, PY};
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::path::PathBuf;

pub(super) struct ImportPlan {
    own_id: &'static str,
    own_path: PathBuf,
    names: HashMap<String, &'static str>,
    eager: BTreeMap<String, BTreeSet<String>>,
    cyclic: BTreeMap<String, BTreeSet<String>>,
}

impl ImportPlan {
    pub(super) fn for_type<T: PY + ?Sized>(own_path: PathBuf) -> Result<Self, ExportError> {
        let mut plan = Self {
            own_id: T::declaration_id(),
            own_path,
            names: T::declaration_names()
                .into_iter()
                .map(|name| (name, T::declaration_id()))
                .collect(),
            eager: BTreeMap::new(),
            cyclic: BTreeMap::new(),
        };
        for dependency in T::dependencies() {
            plan.add(dependency)?;
        }
        Ok(plan)
    }

    pub(super) fn validate_standard_names(&self, imports: &[String]) -> Result<(), ExportError> {
        for line in imports {
            let names = if let Some(module) = line.strip_prefix("import ") {
                module.split('.').next()
            } else {
                line.split_once(" import ").map(|(_, names)| names)
            };
            if let Some(names) = names {
                for name in names.split(", ") {
                    if self.names.contains_key(name) {
                        return Err(ExportError::NameCollision(name.to_owned()));
                    }
                }
            }
        }
        Ok(())
    }

    fn add(&mut self, dep: Dependency) -> Result<(), ExportError> {
        if dep.id == self.own_id {
            return Ok(());
        }
        path::validate(&dep.path)?;
        if self.own_path == dep.path {
            return Err(ExportError::ConflictingFile(self.own_path.clone()));
        }
        if self
            .names
            .insert(dep.name.clone(), dep.id)
            .is_some_and(|id| id != dep.id)
        {
            return Err(ExportError::NameCollision(dep.name));
        }
        let module = path::relative_module(&self.own_path, &dep.path);
        let imports = if reaches_type(&dep, self.own_id, &mut HashSet::new()) {
            &mut self.cyclic
        } else {
            &mut self.eager
        };
        imports.entry(module).or_default().insert(dep.name);
        Ok(())
    }

    pub(super) fn write_eager(&self, out: &mut String) {
        if !self.eager.is_empty() {
            out.push('\n');
        }
        let mut imports = self.eager.iter().collect::<Vec<_>>();
        imports.sort_by(|(left, _), (right, _)| {
            left.to_ascii_lowercase()
                .cmp(&right.to_ascii_lowercase())
                .then(left.cmp(right))
        });
        for (module, names) in imports {
            write_import(out, module, names, "\n");
        }
    }

    pub(super) fn write_cyclic(&self, out: &mut String) {
        for (module, names) in &self.cyclic {
            write_import(out, module, names, "  # noqa: E402 - cyclic dependency\n");
        }
    }
}

fn write_import(out: &mut String, module: &str, names: &BTreeSet<String>, suffix: &str) {
    out.push_str("from ");
    out.push_str(module);
    out.push_str(" import ");
    out.push_str(&names.iter().cloned().collect::<Vec<_>>().join(", "));
    out.push_str(suffix);
}

fn reaches_type(dep: &Dependency, target: &'static str, seen: &mut HashSet<&'static str>) -> bool {
    if dep.id == target {
        return true;
    }
    seen.insert(dep.id)
        && (dep.children)()
            .iter()
            .any(|child| reaches_type(child, target, seen))
}
