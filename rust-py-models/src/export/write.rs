use crate::ir::HEADER;
use crate::ExportError;
use fs2::FileExt;
use std::collections::BTreeMap;
use std::fs::{self, OpenOptions};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

const MANIFEST: &str = ".rust-py-models-manifest";
const MANIFEST_HEADER: &str = "rust-py-models-manifest-v1";
const LOCK: &str = ".rust-py-models-manifest.lock";
static TEMP_NONCE: AtomicU64 = AtomicU64::new(0);

type Manifest = BTreeMap<String, BTreeMap<PathBuf, u64>>;

pub(super) fn modules(
    dir: &Path,
    root_owner: &str,
    outputs: Vec<(PathBuf, String)>,
) -> Result<(), ExportError> {
    fs::create_dir_all(dir)?;
    let lock = OpenOptions::new()
        .create(true)
        .truncate(false)
        .read(true)
        .write(true)
        .open(dir.join(LOCK))?;
    lock.lock_exclusive()?;
    let result = modules_locked(dir, root_owner, outputs);
    let unlock = FileExt::unlock(&lock).map_err(ExportError::Io);
    result.and(unlock)
}

fn modules_locked(
    dir: &Path,
    root_owner: &str,
    outputs: Vec<(PathBuf, String)>,
) -> Result<(), ExportError> {
    validate_portable_paths(dir, &outputs)?;
    let mut manifest = read_manifest(dir)?;
    protect_modified_files(dir, &manifest, &outputs)?;

    for (relative, content) in &outputs {
        module(dir, relative, content)?;
    }

    let previous = manifest.remove(root_owner).unwrap_or_default();
    let current = outputs
        .iter()
        .map(|(path, content)| (path.clone(), content_hash(content.as_bytes())))
        .collect::<BTreeMap<_, _>>();
    for (relative, expected_hash) in previous {
        if current.contains_key(&relative)
            || manifest.values().any(|owned| owned.contains_key(&relative))
        {
            continue;
        }
        let path = dir.join(&relative);
        match fs::read(&path) {
            Ok(existing) => {
                if !existing.starts_with(HEADER.as_bytes())
                    || content_hash(&existing) != expected_hash
                {
                    return Err(ExportError::ConflictingFile(path));
                }
                fs::remove_file(path)?;
            }
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => {}
            Err(err) => return Err(ExportError::Io(err)),
        }
    }
    manifest.insert(root_owner.to_owned(), current);
    write_manifest(dir, &manifest)
}

fn protect_modified_files(
    dir: &Path,
    manifest: &Manifest,
    outputs: &[(PathBuf, String)],
) -> Result<(), ExportError> {
    for (relative, replacement) in outputs {
        let path = dir.join(relative);
        let Ok(existing) = fs::read(&path) else {
            continue;
        };
        if existing == replacement.as_bytes() {
            continue;
        }
        let expected = manifest
            .values()
            .filter_map(|owned| owned.get(relative))
            .any(|hash| *hash == content_hash(&existing));
        if !expected && manifest.values().any(|owned| owned.contains_key(relative)) {
            return Err(ExportError::ConflictingFile(path));
        }
    }
    Ok(())
}

fn validate_portable_paths(dir: &Path, outputs: &[(PathBuf, String)]) -> Result<(), ExportError> {
    let mut paths = BTreeMap::<String, PathBuf>::new();
    for existing in python_files(dir)? {
        paths.insert(portable_key(&existing), existing);
    }
    for (relative, _) in outputs {
        let key = portable_key(relative);
        if let Some(existing) = paths.get(&key) {
            if existing != relative {
                return Err(ExportError::PortablePathCollision {
                    first: existing.clone(),
                    second: relative.clone(),
                });
            }
        } else {
            paths.insert(key, relative.clone());
        }
    }
    Ok(())
}

fn portable_key(path: &Path) -> String {
    path.to_string_lossy()
        .replace('\\', "/")
        .to_ascii_lowercase()
}

fn python_files(dir: &Path) -> Result<Vec<PathBuf>, ExportError> {
    fn visit(root: &Path, directory: &Path, out: &mut Vec<PathBuf>) -> Result<(), ExportError> {
        for entry in fs::read_dir(directory)? {
            let entry = entry?;
            let file_type = entry.file_type()?;
            if file_type.is_dir() {
                visit(root, &entry.path(), out)?;
            } else if file_type.is_file()
                && entry.path().extension().and_then(|x| x.to_str()) == Some("py")
            {
                out.push(
                    entry
                        .path()
                        .strip_prefix(root)
                        .expect("visited path is below export root")
                        .to_path_buf(),
                );
            }
        }
        Ok(())
    }

    let mut out = Vec::new();
    visit(dir, dir, &mut out)?;
    Ok(out)
}

fn module(dir: &Path, relative: &Path, content: &str) -> Result<(), ExportError> {
    let owners = owner_ids(content);
    let path = dir.join(relative);
    match fs::read_to_string(&path) {
        Ok(existing) => {
            let existing_owners = owner_ids(&existing);
            let same_owner = !existing_owners.is_empty()
                && existing_owners.iter().all(|existing| {
                    owners.iter().any(|expected| {
                        existing == expected || existing.starts_with(&format!("{expected}<"))
                    })
                });
            if !existing.starts_with(HEADER) || !same_owner {
                return Err(ExportError::ConflictingFile(path));
            }
            if existing == content {
                return Ok(());
            }
        }
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => {}
        Err(err) => return Err(ExportError::Io(err)),
    }
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    create_package_markers(dir, path.parent())?;
    fs::write(path, content)?;
    Ok(())
}

fn owner_ids(content: &str) -> Vec<&str> {
    let owner = content.lines().nth(1).unwrap_or_default();
    owner
        .strip_prefix("# Rust types: ")
        .map(|items| items.split(", ").collect())
        .or_else(|| owner.strip_prefix("# Rust type: ").map(|item| vec![item]))
        .unwrap_or_default()
}

fn create_package_markers(dir: &Path, mut package: Option<&Path>) -> Result<(), ExportError> {
    while let Some(folder) = package {
        if !folder.starts_with(dir) {
            break;
        }
        let marker = folder.join("__init__.py");
        if !marker.exists() {
            fs::write(&marker, "")?;
        }
        if folder == dir {
            break;
        }
        package = folder.parent();
    }
    Ok(())
}

fn read_manifest(dir: &Path) -> Result<Manifest, ExportError> {
    let path = dir.join(MANIFEST);
    let content = match fs::read_to_string(&path) {
        Ok(content) => content,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => return Ok(Manifest::new()),
        Err(err) => return Err(ExportError::Io(err)),
    };
    let mut lines = content.lines();
    if lines.next() != Some(MANIFEST_HEADER) {
        return Err(ExportError::InvalidManifest(path));
    }
    let mut manifest = Manifest::new();
    for line in lines {
        let mut parts = line.splitn(3, '\t');
        let Some(owner) = parts.next().and_then(decode_hex) else {
            return Err(ExportError::InvalidManifest(path));
        };
        let Some(relative) = parts.next().map(PathBuf::from) else {
            return Err(ExportError::InvalidManifest(path));
        };
        let Some(hash) = parts
            .next()
            .and_then(|value| u64::from_str_radix(value, 16).ok())
        else {
            return Err(ExportError::InvalidManifest(path));
        };
        super::path::validate(&relative).map_err(|_| ExportError::InvalidManifest(path.clone()))?;
        manifest.entry(owner).or_default().insert(relative, hash);
    }
    Ok(manifest)
}

fn write_manifest(dir: &Path, manifest: &Manifest) -> Result<(), ExportError> {
    let mut content = String::from(MANIFEST_HEADER);
    content.push('\n');
    for (owner, files) in manifest {
        let owner = encode_hex(owner.as_bytes());
        for (relative, hash) in files {
            content.push_str(&owner);
            content.push('\t');
            content.push_str(&relative.to_string_lossy());
            content.push('\t');
            content.push_str(&format!("{hash:016x}\n"));
        }
    }
    let nonce = TEMP_NONCE.fetch_add(1, Ordering::Relaxed);
    let temporary = dir.join(format!("{MANIFEST}.{}.{}.tmp", std::process::id(), nonce));
    fs::write(&temporary, content)?;
    fs::rename(&temporary, dir.join(MANIFEST)).map_err(|err| {
        let _ = fs::remove_file(&temporary);
        ExportError::Io(err)
    })
}

fn content_hash(bytes: &[u8]) -> u64 {
    bytes.iter().fold(0xcbf2_9ce4_8422_2325, |hash, byte| {
        (hash ^ u64::from(*byte)).wrapping_mul(0x0000_0100_0000_01b3)
    })
}

fn encode_hex(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        out.push(char::from(HEX[usize::from(byte >> 4)]));
        out.push(char::from(HEX[usize::from(byte & 0x0f)]));
    }
    out
}

fn decode_hex(value: &str) -> Option<String> {
    if value.len() & 1 == 1 {
        return None;
    }
    let bytes = value
        .as_bytes()
        .chunks_exact(2)
        .map(|digits| {
            let high = char::from(digits[0]).to_digit(16)?;
            let low = char::from(digits[1]).to_digit(16)?;
            u8::try_from((high << 4) | low).ok()
        })
        .collect::<Option<Vec<_>>>()?;
    String::from_utf8(bytes).ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    struct TestDir(PathBuf);

    impl TestDir {
        fn new() -> Self {
            let nonce = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("clock before epoch")
                .as_nanos();
            let path = std::env::temp_dir().join(format!(
                "rust-py-models-write-{}-{nonce}",
                std::process::id()
            ));
            fs::create_dir(&path).expect("create test directory");
            Self(path)
        }
    }

    impl Drop for TestDir {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    fn generated(owner: &str, name: &str) -> String {
        format!("{HEADER}# Rust type: {owner}\n# fmt: off\nclass {name}: pass\n")
    }

    #[test]
    fn writes_packages_and_replaces_only_owned_files() {
        let dir = TestDir::new();
        let relative = PathBuf::from("nested/Model.py");
        let generated = generated("tests::Model", "Model");
        modules(
            &dir.0,
            "tests::Root",
            vec![(relative.clone(), generated.clone())],
        )
        .unwrap();
        assert!(dir.0.join("__init__.py").exists());
        assert!(dir.0.join("nested/__init__.py").exists());
        modules(
            &dir.0,
            "tests::Root",
            vec![(relative.clone(), generated.clone())],
        )
        .unwrap();
        assert_eq!(
            fs::read_to_string(dir.0.join(&relative)).unwrap(),
            generated
        );

        fs::write(dir.0.join(&relative), "hand-written Python\n").unwrap();
        assert!(matches!(
            modules(&dir.0, "tests::Root", vec![(relative.clone(), generated)]),
            Err(ExportError::ConflictingFile(_))
        ));
    }

    #[test]
    fn removes_only_obsolete_unshared_owned_files() {
        let dir = TestDir::new();
        let root = PathBuf::from("Root.py");
        let shared = PathBuf::from("Shared.py");
        let root_code = generated("tests::Root", "Root");
        let shared_code = generated("tests::Shared", "Shared");
        modules(
            &dir.0,
            "tests::FirstRoot",
            vec![
                (root.clone(), root_code.clone()),
                (shared.clone(), shared_code.clone()),
            ],
        )
        .unwrap();
        modules(
            &dir.0,
            "tests::SecondRoot",
            vec![(shared.clone(), shared_code)],
        )
        .unwrap();
        modules(&dir.0, "tests::FirstRoot", vec![(root, root_code)]).unwrap();
        assert!(dir.0.join(&shared).is_file());
        modules(&dir.0, "tests::SecondRoot", Vec::new()).unwrap();
        assert!(!dir.0.join(shared).exists());
    }

    #[test]
    fn rejects_case_only_paths() {
        let dir = TestDir::new();
        fs::write(dir.0.join("Model.py"), "hand-written\n").unwrap();
        let result = modules(
            &dir.0,
            "tests::Root",
            vec![(PathBuf::from("model.py"), generated("tests::Root", "Model"))],
        );
        assert!(matches!(
            result,
            Err(ExportError::PortablePathCollision { .. })
        ));
    }
}
