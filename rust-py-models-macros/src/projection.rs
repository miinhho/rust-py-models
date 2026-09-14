use crate::type_expr::type_args;
use std::collections::HashMap;
use syn::Type;

#[derive(Default)]
pub(crate) struct ProjectionMap {
    pub(crate) models: HashMap<String, Vec<bool>>,
}

impl ProjectionMap {
    pub(crate) fn args<'a>(&self, ty: &'a Type) -> Vec<(bool, &'a Type)> {
        let args = type_args(ty);
        let Type::Path(path) = ty else {
            return args.into_iter().map(|arg| (true, arg)).collect();
        };
        if path.qself.is_some()
            || path
                .path
                .segments
                .iter()
                .take(path.path.segments.len().saturating_sub(1))
                .any(|segment| !segment.arguments.is_empty())
        {
            return args.into_iter().map(|arg| (true, arg)).collect();
        }
        let Some(last) = path.path.segments.last() else {
            return args.into_iter().map(|arg| (true, arg)).collect();
        };
        let name = last.ident.to_string();
        let local_model = path.path.segments.len() == 1
            || (path.path.segments.len() == 2 && path.path.segments[0].ident == "self");
        let selected = local_model.then(|| self.models.get(&name)).flatten();
        args.into_iter()
            .enumerate()
            .map(|(index, arg)| {
                let visible = selected.map_or_else(
                    || match name.as_str() {
                        "HashMap" | "IndexMap" => index < 2,
                        "HashSet" | "IndexSet" => index == 0,
                        "DateTime" => false,
                        _ => true,
                    },
                    |slots| slots.get(index).copied().unwrap_or(true),
                );
                (visible, arg)
            })
            .collect()
    }
}
