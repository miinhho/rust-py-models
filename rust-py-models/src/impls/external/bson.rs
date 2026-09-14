use crate::{TypeSpec, PY};

impl PY for ::bson::oid::ObjectId {
    fn type_spec() -> TypeSpec {
        TypeSpec::named("str").with_hashable(true)
    }
}

impl PY for ::bson::Uuid {
    fn type_spec() -> TypeSpec {
        TypeSpec::qualified("uuid", "UUID").with_hashable(true)
    }
}
