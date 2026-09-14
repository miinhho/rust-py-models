#[cfg(feature = "bigdecimal-impl")]
mod bigdecimal;
#[cfg(feature = "bson-impl")]
mod bson;
#[cfg(feature = "bytes-impl")]
mod bytes;
#[cfg(feature = "chrono-impl")]
mod chrono;
#[cfg(feature = "heapless-impl")]
mod heapless;
#[cfg(feature = "indexmap-impl")]
mod indexmap;
#[cfg(feature = "ordered-float-impl")]
mod ordered_float;
#[cfg(feature = "semver-impl")]
mod semver;
#[cfg(feature = "serde-json-impl")]
mod serde_json;
#[cfg(feature = "smol-str-impl")]
mod smol_str;
#[cfg(feature = "tokio-impl")]
mod tokio;
#[cfg(feature = "url-impl")]
mod url;
#[cfg(feature = "uuid-impl")]
mod uuid;
