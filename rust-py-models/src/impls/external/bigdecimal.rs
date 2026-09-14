use crate::{TypeSpec, PY};

impl PY for ::bigdecimal::BigDecimal {
    fn type_spec() -> TypeSpec {
        TypeSpec::qualified("decimal", "Decimal").with_hashable(true)
    }
}
