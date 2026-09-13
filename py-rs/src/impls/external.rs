use crate::PY;

#[cfg(feature = "bytes-impl")]
mod bytes_impl {
    use super::PY;
    impl PY for bytes::Bytes {
        fn name() -> String {
            "bytes".into()
        }
        fn inline() -> String {
            "bytes".into()
        }
        fn is_hashable() -> bool {
            true
        }
    }
    impl PY for bytes::BytesMut {
        fn name() -> String {
            "bytearray".into()
        }
        fn inline() -> String {
            "bytearray".into()
        }
    }
}

#[cfg(feature = "chrono-impl")]
mod chrono_impl {
    use super::PY;
    macro_rules! date_type {
        ($($ty:ty => $python:expr),* $(,)?) => {$ (
            impl PY for $ty {
                fn name() -> String { $python.into() }
                fn inline() -> String { $python.into() }
                fn annotation_imports() -> Vec<String> { vec!["import datetime".into()] }
                fn is_hashable() -> bool { true }
            }
        )*};
    }
    date_type!(
        chrono::NaiveDate => "datetime.date",
        chrono::NaiveTime => "datetime.time",
        chrono::NaiveDateTime => "datetime.datetime"
    );
    impl<Tz: chrono::TimeZone> PY for chrono::DateTime<Tz> {
        fn name() -> String {
            "datetime.datetime".into()
        }
        fn inline() -> String {
            "datetime.datetime".into()
        }
        fn annotation_imports() -> Vec<String> {
            vec!["import datetime".into()]
        }
        fn is_hashable() -> bool {
            true
        }
    }
}

#[cfg(feature = "serde-json-impl")]
mod serde_json_impl {
    use super::PY;
    impl PY for serde_json::Value {
        fn name() -> String {
            "_PyRsJsonValue".into()
        }
        fn inline() -> String {
            "_PyRsJsonValue".into()
        }
        fn annotation_imports() -> Vec<String> {
            vec!["from typing import TypeAlias".into()]
        }
        fn annotation_definitions() -> Vec<String> {
            vec!["_PyRsJsonValue: TypeAlias = None | bool | int | float | str | list[\"_PyRsJsonValue\"] | dict[str, \"_PyRsJsonValue\"]".into()]
        }
    }
    impl PY for serde_json::Number {
        fn name() -> String {
            "int | float".into()
        }
        fn inline() -> String {
            "int | float".into()
        }
        fn is_hashable() -> bool {
            true
        }
    }
    impl PY for serde_json::Map<String, serde_json::Value> {
        fn name() -> String {
            "dict[str, _PyRsJsonValue]".into()
        }
        fn inline() -> String {
            "dict[str, _PyRsJsonValue]".into()
        }
        fn annotation_imports() -> Vec<String> {
            <serde_json::Value as PY>::annotation_imports()
        }
        fn annotation_definitions() -> Vec<String> {
            <serde_json::Value as PY>::annotation_definitions()
        }
    }
}

#[cfg(feature = "url-impl")]
mod url_impl {
    use super::PY;
    impl PY for url::Url {
        fn name() -> String {
            "str".into()
        }
        fn inline() -> String {
            "str".into()
        }
        fn is_hashable() -> bool {
            true
        }
    }
}

#[cfg(feature = "uuid-impl")]
mod uuid_impl {
    use super::PY;
    impl PY for uuid::Uuid {
        fn name() -> String {
            "uuid.UUID".into()
        }
        fn inline() -> String {
            "uuid.UUID".into()
        }
        fn annotation_imports() -> Vec<String> {
            vec!["import uuid".into()]
        }
        fn is_hashable() -> bool {
            true
        }
    }
}
