use super::{Imports, Models, SourceKey};
use quote::quote;
use std::collections::{BTreeMap, HashSet};
use std::path::{Path, PathBuf};
use syn::parse::Parser;
use syn::punctuated::Punctuated;
use syn::{Attribute, DeriveInput, Item, Path as SynPath, Token};

pub(super) struct Package {
    pub(super) models: Models,
    pub(super) imports: Imports,
    pub(super) locations: BTreeMap<String, SourceKey>,
}

pub(super) fn scan_package() -> Package {
    let Some(manifest) = std::env::var_os("CARGO_MANIFEST_DIR").map(PathBuf::from) else {
        return Package {
            models: Models::new(),
            imports: Imports::new(),
            locations: BTreeMap::new(),
        };
    };
    let mut scanner = Scanner::default();
    for root in [manifest.join("src/lib.rs"), manifest.join("src/main.rs")] {
        scanner.scan_file(&root, root.display().to_string());
    }
    for directory in ["tests", "examples", "benches"] {
        let Ok(entries) = std::fs::read_dir(manifest.join(directory)) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().is_some_and(|extension| extension == "rs") {
                scanner.scan_file(&path, path.display().to_string());
            }
        }
    }
    let models = scanner
        .scopes
        .into_iter()
        .flat_map(|(scope, inputs)| {
            inputs
                .into_iter()
                .map(move |(name, input)| (format!("{scope}::{name}"), (scope.clone(), input)))
        })
        .collect();
    Package {
        models,
        imports: scanner.imports,
        locations: scanner.locations,
    }
}

#[derive(Default)]
struct Scanner {
    scopes: BTreeMap<String, BTreeMap<String, DeriveInput>>,
    imports: BTreeMap<String, Vec<syn::ItemUse>>,
    locations: BTreeMap<String, SourceKey>,
    seen: HashSet<PathBuf>,
}

impl Scanner {
    fn scan_file(&mut self, path: &Path, scope: String) {
        let Ok(path) = path.canonicalize() else {
            return;
        };
        if !self.seen.insert(path.clone()) {
            return;
        }
        let Ok(source) = std::fs::read_to_string(&path) else {
            return;
        };
        let Ok(file) = syn::parse_file(&source) else {
            return;
        };
        let searchable = mask_comments_and_strings(&source);
        let mut cursor = 0;
        let Some(module_dir) = module_directory(&path) else {
            return;
        };
        self.scan_items(
            &file.items,
            &searchable,
            &mut cursor,
            &path,
            &module_dir,
            scope,
        );
    }

    fn scan_items(
        &mut self,
        items: &[Item],
        source: &str,
        cursor: &mut usize,
        file: &Path,
        module_dir: &Path,
        scope: String,
    ) {
        for item in items {
            match item {
                Item::Use(item) => {
                    self.imports
                        .entry(scope.clone())
                        .or_default()
                        .push(item.clone());
                }
                Item::Struct(item) => {
                    let location =
                        locate_declaration(source, cursor, "struct", &item.ident.to_string());
                    if has_py_derive(&item.attrs) {
                        self.record_model(
                            &scope,
                            file,
                            location,
                            syn::parse2::<DeriveInput>(quote!(#item)),
                        );
                    }
                }
                Item::Enum(item) => {
                    let location =
                        locate_declaration(source, cursor, "enum", &item.ident.to_string());
                    if has_py_derive(&item.attrs) {
                        self.record_model(
                            &scope,
                            file,
                            location,
                            syn::parse2::<DeriveInput>(quote!(#item)),
                        );
                    }
                }
                Item::Mod(module) => {
                    locate_declaration(source, cursor, "mod", &module.ident.to_string());
                    let child_scope = format!("{scope}::{}", module.ident);
                    if let Some((_, children)) = &module.content {
                        self.scan_items(
                            children,
                            source,
                            cursor,
                            file,
                            &module_dir.join(module.ident.to_string()),
                            child_scope,
                        );
                    } else if let Some(path) =
                        module_path(file, module_dir, &module.ident.to_string(), &module.attrs)
                    {
                        self.scan_file(&path, child_scope);
                    }
                }
                _ => {}
            }
        }
    }

    fn record_model(
        &mut self,
        scope: &str,
        file: &Path,
        location: Option<(usize, usize)>,
        input: syn::Result<DeriveInput>,
    ) {
        let (Some((line, column)), Ok(input)) = (location, input) else {
            return;
        };
        let id = format!("{scope}::{}", input.ident);
        self.locations.insert(
            id,
            SourceKey {
                file: file.to_path_buf(),
                line,
                column,
            },
        );
        self.scopes
            .entry(scope.to_owned())
            .or_default()
            .insert(input.ident.to_string(), input);
    }
}

fn mask_comments_and_strings(source: &str) -> String {
    let mut bytes = source.as_bytes().to_vec();
    let mut index = 0;
    while index < bytes.len() {
        let start = index;
        if bytes[index..].starts_with(b"//") {
            index += 2;
            while index < bytes.len() && bytes[index] != b'\n' {
                index += 1;
            }
        } else if bytes[index..].starts_with(b"/*") {
            index += 2;
            let mut depth = 1;
            while index < bytes.len() && depth > 0 {
                if bytes[index..].starts_with(b"/*") {
                    depth += 1;
                    index += 2;
                } else if bytes[index..].starts_with(b"*/") {
                    depth -= 1;
                    index += 2;
                } else {
                    index += 1;
                }
            }
        } else if bytes[index] == b'"' {
            index += 1;
            while index < bytes.len() {
                if bytes[index] == b'\\' {
                    index = (index + 2).min(bytes.len());
                } else if bytes[index] == b'"' {
                    index += 1;
                    break;
                } else {
                    index += 1;
                }
            }
        } else if let Some((quote, hashes)) = raw_string_start(&bytes, index) {
            index = quote + 1;
            while index < bytes.len() {
                if bytes[index] == b'"'
                    && bytes.get(index + 1..index + 1 + hashes)
                        == Some(&bytes[quote - hashes..quote])
                {
                    index += 1 + hashes;
                    break;
                }
                index += 1;
            }
        } else {
            index += 1;
            continue;
        }
        for byte in &mut bytes[start..index] {
            if *byte != b'\n' {
                *byte = b' ';
            }
        }
    }
    String::from_utf8(bytes).expect("masking preserves UTF-8")
}

fn raw_string_start(bytes: &[u8], index: usize) -> Option<(usize, usize)> {
    let mut cursor = index;
    if bytes.get(cursor) == Some(&b'b') {
        cursor += 1;
    }
    if bytes.get(cursor) != Some(&b'r') {
        return None;
    }
    cursor += 1;
    let hashes = cursor;
    while bytes.get(cursor) == Some(&b'#') {
        cursor += 1;
    }
    (bytes.get(cursor) == Some(&b'"')).then_some((cursor, cursor - hashes))
}

fn locate_declaration(
    source: &str,
    cursor: &mut usize,
    kind: &str,
    name: &str,
) -> Option<(usize, usize)> {
    let bytes = source.as_bytes();
    let mut from = *cursor;
    while let Some(relative) = source[from..].find(kind) {
        let start = from + relative;
        let before = start
            .checked_sub(1)
            .and_then(|index| bytes.get(index))
            .copied();
        let after = bytes.get(start + kind.len()).copied();
        if before.is_some_and(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
            || !after.is_some_and(|byte| byte.is_ascii_whitespace())
        {
            from = start + kind.len();
            continue;
        }
        let mut ident = start + kind.len();
        while bytes.get(ident).is_some_and(u8::is_ascii_whitespace) {
            ident += 1;
        }
        if source[ident..].starts_with(name)
            && !bytes
                .get(ident + name.len())
                .is_some_and(|byte| byte.is_ascii_alphanumeric() || *byte == b'_')
        {
            *cursor = ident + name.len();
            let prefix = &source[..ident];
            let line = prefix.bytes().filter(|byte| *byte == b'\n').count() + 1;
            let column = ident - prefix.rfind('\n').map_or(0, |index| index + 1) + 1;
            return Some((line, column));
        }
        from = start + kind.len();
    }
    None
}

fn module_directory(file: &Path) -> Option<PathBuf> {
    let parent = file.parent()?;
    if matches!(file.file_name()?.to_str()?, "lib.rs" | "main.rs" | "mod.rs") {
        Some(parent.to_path_buf())
    } else {
        Some(parent.join(file.file_stem()?))
    }
}
fn module_path(file: &Path, module_dir: &Path, name: &str, attrs: &[Attribute]) -> Option<PathBuf> {
    let parent = file.parent()?;
    for attr in attrs {
        if attr.path().is_ident("path") {
            if let syn::Meta::NameValue(value) = &attr.meta {
                if let syn::Expr::Lit(syn::ExprLit {
                    lit: syn::Lit::Str(path),
                    ..
                }) = &value.value
                {
                    return Some(parent.join(path.value()));
                }
            }
        }
    }
    let direct = module_dir.join(format!("{name}.rs"));
    if direct.exists() {
        Some(direct)
    } else {
        Some(module_dir.join(name).join("mod.rs"))
    }
}

fn has_py_derive(attrs: &[Attribute]) -> bool {
    attrs
        .iter()
        .filter(|attr| attr.path().is_ident("derive"))
        .any(|attr| {
            let Ok(list) = attr.meta.require_list() else {
                return false;
            };
            let Ok(paths) =
                Punctuated::<SynPath, Token![,]>::parse_terminated.parse2(list.tokens.clone())
            else {
                return false;
            };
            paths.iter().any(|path| {
                path.segments
                    .last()
                    .is_some_and(|segment| segment.ident == "PY")
            })
        })
}

#[cfg(test)]
mod tests {
    use super::{locate_declaration, mask_comments_and_strings};

    #[test]
    fn declaration_locations_ignore_comments_and_strings() {
        let source = r###"
// struct Model;
const TEXT: &str = "enum Model";
const RAW: &str = r#"struct Model"#;
#[derive(PY)]
struct Model<T> { value: T }
"###;
        let searchable = mask_comments_and_strings(source);
        let mut cursor = 0;

        assert_eq!(
            locate_declaration(&searchable, &mut cursor, "struct", "Model"),
            Some((6, 8))
        );
    }
}
