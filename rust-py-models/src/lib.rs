//! Generate Python model declarations from Rust types.
//!
//! Derive [`PY`] on a struct or enum, then call [`PY::export_all`] or add
//! `#[py(export)]` to generate bindings during `cargo test`.

mod dependency;
mod error;
mod export;
mod hashability;
mod impls;
mod ir;
mod trait_py;

pub use dependency::Dependency;
pub use error::ExportError;
pub use ir::{
    DataclassOptions, DataclassSpec, Declaration, EnumMemberSpec, FieldDefault, FieldSpec,
    ModelSpec, StringEnumSpec, TypeExpr, TypeSpec,
};
pub use rust_py_models_macros::PY;
pub use trait_py::PY;

#[doc(hidden)]
pub mod __private {
    pub use crate::hashability::check as check_hashability;
}
