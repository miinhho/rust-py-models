use crate::{TypeSpec, PY};

impl PY for ::uuid::Uuid {
    fn type_spec() -> TypeSpec {
        TypeSpec::qualified("uuid", "UUID").with_hashable(true)
    }
}
