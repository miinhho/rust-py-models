use crate::{TypeSpec, PY};

impl PY for ::url::Url {
    fn type_spec() -> TypeSpec {
        TypeSpec::named("str").with_hashable(true)
    }
}
