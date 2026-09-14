use crate::{TypeSpec, PY};

impl PY for ::smol_str::SmolStr {
    fn type_spec() -> TypeSpec {
        TypeSpec::named("str").with_hashable(true)
    }
}
