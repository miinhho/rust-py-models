use super::{Imports, Masks, Models};
use crate::attrs::options;
use crate::fields::{project_field, FieldProjection};
use crate::generics::visible_params;
use crate::projection::ProjectionMap;
use crate::type_expr::type_args;
use std::collections::{BTreeMap, HashMap, HashSet};
use syn::{Data, DeriveInput, GenericParam};

pub(super) fn resolve(models: &Models, imports: &Imports) -> syn::Result<Masks> {
    let mut masks = models
        .iter()
        .map(|(id, (_, input))| {
            (
                id.clone(),
                vec![false; input.generics.type_params().count()],
            )
        })
        .collect::<Masks>();
    converge(models, imports, &mut masks)?;

    // Retain legacy generic parameters in a cycle with no value-bearing field.
    let edges = models
        .iter()
        .map(|(id, (scope, input))| {
            let aliases = alias_targets(scope, models, imports);
            (id.clone(), referenced_models(input, &aliases))
        })
        .collect::<BTreeMap<_, _>>();
    let mut preserved = false;
    for id in models.keys() {
        let component = models
            .keys()
            .filter(|candidate| {
                reachable(id, candidate, &edges) && reachable(candidate, id, &edges)
            })
            .collect::<Vec<_>>();
        let cyclic = component.len() > 1
            || edges
                .get(id)
                .is_some_and(|neighbors| neighbors.contains(id));
        let has_visible = component
            .iter()
            .any(|member| masks[*member].iter().any(|visible| *visible));
        if cyclic && !has_visible {
            for member in component {
                masks.get_mut(member).expect("registered model").fill(true);
                preserved = true;
            }
        }
    }
    if preserved {
        converge(models, imports, &mut masks)?;
    }
    Ok(masks)
}

fn converge(models: &Models, imports: &Imports, masks: &mut Masks) -> syn::Result<()> {
    loop {
        let mut changed = false;
        for (id, (scope, input)) in models {
            if matches!(input.data, Data::Union(_)) {
                continue;
            }
            let projection = projection_for(scope, models, masks, imports);
            let declared = input
                .generics
                .type_params()
                .map(|param| param.ident.to_string())
                .collect::<HashSet<_>>();
            let attr = options(&input.attrs)?;
            let visible = visible_params(&input.data, &attr, &declared, &projection)?;
            let concrete = attr
                .concrete
                .iter()
                .map(|(ident, _)| ident.to_string())
                .collect::<HashSet<_>>();
            let slots = input
                .generics
                .params
                .iter()
                .filter_map(|param| match param {
                    GenericParam::Type(param) => Some(param.ident.to_string()),
                    _ => None,
                })
                .map(|param| visible.contains(&param) && !concrete.contains(&param))
                .collect::<Vec<_>>();
            let current = masks.get_mut(id).expect("registered model");
            if *current != slots {
                *current = slots;
                changed = true;
            }
        }
        if !changed {
            return Ok(());
        }
    }
}

pub(super) fn projection_for(
    scope: &str,
    models: &Models,
    masks: &Masks,
    imports: &Imports,
) -> ProjectionMap {
    let targets = alias_targets(scope, models, imports);
    ProjectionMap {
        models: targets
            .into_iter()
            .map(|(alias, id)| (alias, masks[&id].clone()))
            .collect(),
    }
}

fn alias_targets(scope: &str, models: &Models, imports: &Imports) -> HashMap<String, String> {
    let (root, current) = split_scope(scope);
    let mut aliases = HashMap::new();
    for (id, (target_scope, input)) in models {
        let (target_root, target) = split_scope(target_scope);
        if root != target_root {
            continue;
        }
        let name = input.ident.to_string();
        let mut crate_path = vec!["crate"];
        crate_path.extend(target.iter().copied());
        crate_path.push(&name);
        aliases.insert(crate_path.join("::"), id.clone());
        for up in 0..=current.len() {
            let ancestor = &current[..current.len() - up];
            if !target.starts_with(ancestor) {
                continue;
            }
            let mut relative = vec!["super"; up];
            relative.extend(target[ancestor.len()..].iter().copied());
            relative.push(&name);
            aliases.insert(relative.join("::"), id.clone());
            if up == 0 && target == current {
                aliases.insert(format!("self::{name}"), id.clone());
            }
        }
    }
    if let Some(uses) = imports.get(scope) {
        let mut imported = Vec::new();
        for item in uses {
            collect_imports(&item.tree, Vec::new(), &mut imported);
        }
        for _ in 0..imported.len() {
            let mut changed = false;
            for (alias, path) in &imported {
                for (candidate, id) in aliases.clone() {
                    if candidate == *path || candidate.starts_with(&format!("{path}::")) {
                        let key = format!("{alias}{}", &candidate[path.len()..]);
                        if aliases.insert(key, id.clone()).as_ref() != Some(&id) {
                            changed = true;
                        }
                    }
                }
            }
            if !changed {
                break;
            }
        }
    }
    aliases
}

fn collect_imports(tree: &syn::UseTree, mut prefix: Vec<String>, into: &mut Vec<(String, String)>) {
    match tree {
        syn::UseTree::Path(path) => {
            prefix.push(path.ident.to_string());
            collect_imports(&path.tree, prefix, into);
        }
        syn::UseTree::Name(name) => {
            let ident = name.ident.to_string();
            prefix.push(ident.clone());
            into.push((ident, prefix.join("::")));
        }
        syn::UseTree::Rename(rename) => {
            prefix.push(rename.ident.to_string());
            into.push((rename.rename.to_string(), prefix.join("::")));
        }
        syn::UseTree::Group(group) => {
            for item in &group.items {
                collect_imports(item, prefix.clone(), into);
            }
        }
        syn::UseTree::Glob(_) => {}
    }
}

fn split_scope(scope: &str) -> (&str, Vec<&str>) {
    match scope.split_once("::") {
        Some((root, modules)) => (root, modules.split("::").collect()),
        None => (scope, Vec::new()),
    }
}

fn referenced_models(input: &DeriveInput, aliases: &HashMap<String, String>) -> HashSet<String> {
    let mut references = HashSet::new();
    let mut visit_fields = |fields: &syn::Fields| {
        for field in fields {
            if let Ok(attr) = options(&field.attrs) {
                if let FieldProjection::Rust(ty) = project_field(field, &attr) {
                    collect_references(ty, aliases, &mut references);
                }
            }
        }
    };
    match &input.data {
        Data::Struct(data) => visit_fields(&data.fields),
        Data::Enum(data) => {
            for variant in &data.variants {
                visit_fields(&variant.fields);
            }
        }
        Data::Union(_) => {}
    }
    references
}

fn collect_references(
    ty: &syn::Type,
    aliases: &HashMap<String, String>,
    into: &mut HashSet<String>,
) {
    if let syn::Type::Path(path) = ty {
        let name = path
            .path
            .segments
            .iter()
            .map(|segment| segment.ident.to_string())
            .collect::<Vec<_>>()
            .join("::");
        if let Some(id) = aliases.get(&name) {
            into.insert(id.clone());
        }
    }
    for arg in type_args(ty) {
        collect_references(arg, aliases, into);
    }
}

fn reachable(from: &str, to: &str, edges: &BTreeMap<String, HashSet<String>>) -> bool {
    let mut pending = edges
        .get(from)
        .into_iter()
        .flat_map(|neighbors| neighbors.iter())
        .collect::<Vec<_>>();
    let mut seen = HashSet::new();
    while let Some(current) = pending.pop() {
        if current == to {
            return true;
        }
        if seen.insert(current) {
            pending.extend(
                edges
                    .get(current)
                    .into_iter()
                    .flat_map(|neighbors| neighbors.iter()),
            );
        }
    }
    false
}
