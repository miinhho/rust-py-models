use crate::{TypeSpec, PY};

impl PY for ::bytes::Bytes {
    fn type_spec() -> TypeSpec {
        TypeSpec::named("bytes").with_hashable(true)
    }
}

impl PY for ::bytes::BytesMut {
    fn type_spec() -> TypeSpec {
        TypeSpec::named("bytearray")
    }
}
