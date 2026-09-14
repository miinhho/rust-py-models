use crate::{Declaration, TypeSpec, PY};

fn json_value_spec() -> TypeSpec {
    let reference = || TypeSpec::unsafe_raw("\"_PyRsJsonValue\"");
    let target = TypeSpec::union(vec![
        TypeSpec::named("None").with_hashable(true),
        TypeSpec::named("bool").with_hashable(true),
        TypeSpec::named("int").with_hashable(true),
        TypeSpec::named("float").with_hashable(true),
        TypeSpec::named("str").with_hashable(true),
        TypeSpec::list(reference()),
        TypeSpec::dict(TypeSpec::named("str").with_hashable(true), reference()),
    ]);
    TypeSpec::named("_PyRsJsonValue").with_definition(Declaration::TypeAlias {
        name: "_PyRsJsonValue".into(),
        target,
        documentation: None,
    })
}

impl PY for ::serde_json::Value {
    fn type_spec() -> TypeSpec {
        json_value_spec()
    }
}

impl PY for ::serde_json::Number {
    fn type_spec() -> TypeSpec {
        TypeSpec::union(vec![
            TypeSpec::named("int").with_hashable(true),
            TypeSpec::named("float").with_hashable(true),
        ])
    }
}

impl PY for ::serde_json::Map<String, ::serde_json::Value> {
    fn type_spec() -> TypeSpec {
        TypeSpec::dict(
            TypeSpec::named("str").with_hashable(true),
            json_value_spec(),
        )
    }
}
