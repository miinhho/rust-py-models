use crate::type_expr::type_args;
use std::collections::HashMap;
use syn::{GenericArgument, PathArguments, Type};

#[derive(Clone, Default)]
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
        let path_name = path
            .path
            .segments
            .iter()
            .map(|segment| segment.ident.to_string())
            .collect::<Vec<_>>()
            .join("::");
        let selected = self.models.get(&path_name);
        args.into_iter()
            .enumerate()
            .map(|(index, arg)| {
                let visible = selected.map_or_else(
                    || match name.as_str() {
                        "Result" if is_result_path(path) => index == 0,
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

fn is_result_path(path: &syn::TypePath) -> bool {
    let segments = path.path.segments.iter().collect::<Vec<_>>();
    let path_matches = match segments.as_slice() {
        [result] => result.ident == "Result",
        [root, module, result] => {
            (root.ident == "std" || root.ident == "core")
                && module.ident == "result"
                && result.ident == "Result"
        }
        _ => false,
    };
    if !path_matches {
        return false;
    }
    let Some(last) = segments.last() else {
        return false;
    };
    let PathArguments::AngleBracketed(arguments) = &last.arguments else {
        return false;
    };
    matches!(
        arguments.args.iter().collect::<Vec<_>>().as_slice(),
        [GenericArgument::Type(_), GenericArgument::Type(_)]
    )
}
