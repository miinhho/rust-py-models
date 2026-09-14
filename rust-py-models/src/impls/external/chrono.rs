use crate::{TypeSpec, PY};

impl PY for ::chrono::NaiveDate {
    fn type_spec() -> TypeSpec {
        TypeSpec::qualified("datetime", "date").with_hashable(true)
    }
}

impl PY for ::chrono::NaiveTime {
    fn type_spec() -> TypeSpec {
        TypeSpec::qualified("datetime", "time").with_hashable(true)
    }
}

impl PY for ::chrono::NaiveDateTime {
    fn type_spec() -> TypeSpec {
        TypeSpec::qualified("datetime", "datetime").with_hashable(true)
    }
}

impl<Tz: ::chrono::TimeZone> PY for ::chrono::DateTime<Tz> {
    fn type_spec() -> TypeSpec {
        TypeSpec::qualified("datetime", "datetime").with_hashable(true)
    }
}
