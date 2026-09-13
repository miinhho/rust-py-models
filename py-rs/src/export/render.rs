use super::imports::ImportPlan;
use super::path;
use super::write::HEADER;
use crate::{ExportError, PY};
use std::collections::BTreeSet;

pub(crate) fn render<T: PY + ?Sized>() -> Result<String, ExportError> {
    T::validate()?;
    let own = T::output_path().ok_or(ExportError::NotExportable(std::any::type_name::<T>()))?;
    path::validate(&own)?;
    let imports = ImportPlan::for_type::<T>(own)?;

    let mut standard_imports = T::prelude()
        .lines()
        .map(str::to_owned)
        .chain(T::imports())
        .collect::<Vec<_>>();
    standard_imports.sort();
    standard_imports.dedup();
    standard_imports.sort_by_key(|line| (line.starts_with("from "), line.clone()));
    imports.validate_standard_names(&standard_imports)?;

    let mut out = format!(
        "{HEADER}# Rust type: {}\nfrom __future__ import annotations\n\n",
        T::declaration_id()
    );
    for line in standard_imports {
        out.push_str(&line);
        out.push('\n');
    }
    imports.write_eager(&mut out);

    let definitions = T::definitions().into_iter().collect::<BTreeSet<_>>();
    if definitions.is_empty() {
        out.push_str("\n\n");
    } else {
        out.push('\n');
        out.push_str(&definitions.into_iter().collect::<Vec<_>>().join("\n"));
        out.push_str("\n\n");
    }
    out.push_str(&T::decl());
    out.push('\n');
    imports.write_cyclic(&mut out);
    Ok(out)
}
