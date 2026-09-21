use crate::projection::ProjectionMap;
use std::collections::{BTreeMap, HashMap, HashSet};
use std::path::PathBuf;
use std::sync::OnceLock;
use syn::DeriveInput;

mod resolve;
mod source;

use resolve::{projection_for, resolve};

#[derive(Clone)]
pub(crate) struct Decision {
    pub(crate) projection: ProjectionMap,
    pub(crate) visible: HashSet<String>,
}

#[derive(Clone, Eq, Hash, PartialEq)]
pub(super) struct SourceKey {
    pub(super) file: PathBuf,
    pub(super) line: usize,
    pub(super) column: usize,
}

type Decisions = HashMap<SourceKey, Decision>;
pub(super) type Models = BTreeMap<String, (String, DeriveInput)>;
pub(super) type Masks = BTreeMap<String, Vec<bool>>;
pub(super) type Imports = BTreeMap<String, Vec<syn::ItemUse>>;

static DECISIONS: OnceLock<Decisions> = OnceLock::new();

pub(crate) fn decision(input: &DeriveInput) -> Option<&'static Decision> {
    let span = input.ident.span().unwrap();
    let key = SourceKey {
        file: span.local_file()?.canonicalize().ok()?,
        line: span.line(),
        column: span.column(),
    };
    DECISIONS.get_or_init(build_decisions).get(&key)
}

fn build_decisions() -> Decisions {
    let package = source::scan_package();
    let mut decisions = Decisions::new();
    if let Ok(masks) = resolve(&package.models, &package.imports) {
        for (id, (scope, input)) in &package.models {
            let slots = &masks[id];
            let visible = input
                .generics
                .type_params()
                .zip(slots)
                .filter(|(_, used)| **used)
                .map(|(param, _)| param.ident.to_string())
                .collect();
            let decision = Decision {
                projection: projection_for(scope, &package.models, &masks, &package.imports),
                visible,
            };
            decisions.insert(package.locations[id].clone(), decision);
        }
    }
    decisions
}
