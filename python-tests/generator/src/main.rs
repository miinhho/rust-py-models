//! Generate the Python fixtures consumed by the repository's Python tests.

#[path = "../../../rust-py-models/tests/python/enum_mapping.rs"]
mod enum_mapping;
#[path = "../../../rust-py-models/tests/python/export.rs"]
mod export;
#[path = "../../../rust-py-models/tests/python/generics.rs"]
mod generics;
#[path = "../../../rust-py-models/tests/python/serde_compat.rs"]
mod serde_compat;
#[path = "../../../rust-py-models/tests/python/types.rs"]
mod types;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args_os().skip(1);
    let dir = args.next().ok_or_else(|| {
        std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "usage: rust-py-models-test-generator <output-directory>",
        )
    })?;
    if args.next().is_some() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "usage: rust-py-models-test-generator <output-directory>",
        )
        .into());
    }

    rust_py_models::export_all_to(dir)?;
    Ok(())
}
