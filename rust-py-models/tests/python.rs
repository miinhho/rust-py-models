#[path = "python/enum_mapping.rs"]
mod enum_mapping;
#[path = "python/export.rs"]
mod export;
#[path = "python/generics.rs"]
mod generics;
#[path = "python/serde_compat.rs"]
mod serde_compat;
#[path = "python/types.rs"]
mod types;

#[test]
fn registered_roots_export_to_explicit_directory() {
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("clock before Unix epoch")
        .as_nanos();
    let dir = std::env::temp_dir().join(format!(
        "rust-py-models-registered-{}-{nonce}",
        std::process::id()
    ));

    rust_py_models::export_all_to(&dir).unwrap();
    for path in [
        "User.py",
        "Root.py",
        "TypeCoverage.py",
        "auth/Event.py",
        "graph/left/NestedCycleA.py",
    ] {
        assert!(dir.join(path).is_file(), "missing {path}");
    }
    #[cfg(feature = "serde-compat")]
    assert!(dir.join("SerdeEnvelope.py").is_file());
    std::fs::remove_dir_all(dir).unwrap();
}
