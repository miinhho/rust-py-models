mod collections;
mod primitives;
mod tuples;
mod wrappers;

#[cfg(any(
    feature = "bytes-impl",
    feature = "chrono-impl",
    feature = "serde-json-impl",
    feature = "url-impl",
    feature = "uuid-impl"
))]
mod external;
