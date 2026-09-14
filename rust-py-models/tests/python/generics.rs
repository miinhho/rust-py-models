#![allow(dead_code)]

#[cfg(test)]
use rust_py_models::ExportError;
use rust_py_models::PY;
use std::collections::HashSet;
use std::marker::PhantomData;
use std::path::PathBuf;

#[derive(PY)]
struct Item {
    id: u64,
}

#[derive(PY)]
struct Page<T> {
    value: T,
    items: Vec<Option<T>>,
    marker: PhantomData<T>,
}

#[derive(PY)]
struct Pair<A, B> {
    left: A,
    right: B,
}

#[derive(PY)]
struct Borrowed<'a> {
    label: &'a str,
    item: &'a Item,
}

#[derive(PY)]
enum Choice<T> {
    One(T),
    Many { items: Vec<T> },
}

#[derive(PY)]
#[py(export)]
struct Root {
    page: Page<Item>,
    paths: Page<PathBuf>,
    choice: Choice<Item>,
    borrowed: Borrowed<'static>,
    pair: Pair<PathBuf, Item>,
}

#[derive(PY)]
struct GenericSet<T> {
    items: HashSet<T>,
}

#[derive(PY)]
struct MixedSets {
    valid: GenericSet<u64>,
    invalid: GenericSet<Item>,
}

struct NotPython;

#[derive(PY)]
#[py(concrete(T = String))]
struct ConcretePage<T, U> {
    fixed: T,
    dynamic: U,
}

#[derive(PY)]
#[py(bound = "T: rust_py_models::PY")]
struct ExplicitBound<T> {
    value: T,
}

#[derive(PY)]
struct FieldAs {
    #[py(as = "String")]
    numeric_id: u64,
}

#[derive(PY)]
#[py(as = "String")]
struct TransparentId(u64);

#[derive(PY)]
#[py(as = "T")]
struct GenericTransparent<T>(T);

#[derive(PY)]
#[py(newtype)]
struct UserId(u64);

#[derive(PY)]
#[py(export)]
struct NewTypeHolder {
    user_id: UserId,
}

#[derive(PY)]
struct GenericNode<T> {
    value: T,
    children: Vec<Box<GenericNode<T>>>,
}

#[derive(PY)]
struct GenericLeft<T> {
    right: Option<Box<GenericRight<T>>>,
    marker: PhantomData<T>,
}

#[derive(PY)]
struct GenericRight<T> {
    left: Option<Box<GenericLeft<T>>>,
    marker: PhantomData<T>,
}

#[derive(PY)]
struct DefaultPage<T = String> {
    value: T,
}

#[derive(PY)]
#[py(export)]
struct GenericEdgeRoot {
    node: GenericNode<Item>,
    left: GenericLeft<Item>,
    default_page: DefaultPage,
}

#[test]
fn generic_declarations_keep_symbolic_parameters() {
    let page = Page::<Item>::export_to_string().unwrap();
    assert!(
        page.contains("from typing import Generic, TypeVar"),
        "{page}"
    );
    assert!(page.contains("T = TypeVar(\"T\")"), "{page}");
    assert!(page.contains("class Page(Generic[T]):"), "{page}");
    assert!(page.contains("value: T"), "{page}");
    assert!(page.contains("items: list[T | None]"), "{page}");
    assert!(!page.contains("marker:"), "{page}");
    assert!(!page.contains("from .Item import Item"), "{page}");
    assert_eq!(Page::<PathBuf>::export_to_string().unwrap(), page);

    let root = Root::export_to_string().unwrap();
    assert!(root.contains("page: Page[Item]"), "{root}");
    assert!(root.contains("paths: Page[pathlib.Path]"), "{root}");
    assert!(root.contains("choice: Choice[Item]"), "{root}");
    assert!(root.contains("borrowed: Borrowed"), "{root}");
    assert!(root.contains("pair: Pair[pathlib.Path, Item]"), "{root}");
    assert!(root.contains("import pathlib"), "{root}");

    let choice = Choice::<Item>::export_to_string().unwrap();
    assert!(choice.contains("class ChoiceOne(Generic[T]):"), "{choice}");
    assert!(
        choice.contains("Choice: TypeAlias = ChoiceOne[T] | ChoiceMany[T]"),
        "{choice}"
    );
    assert!(Borrowed::<'static>::export_to_string()
        .unwrap()
        .contains("label: str"));
    let pair = Pair::<PathBuf, Item>::export_to_string().unwrap();
    assert!(pair.contains("class Pair(Generic[A, B]):"), "{pair}");
    assert!(pair.contains("left: A"), "{pair}");
    assert!(pair.contains("right: B"), "{pair}");
}

#[test]
fn exports_generic_graph() {
    let dir = std::env::temp_dir().join(format!("rust-py-models-generics-{}", std::process::id()));
    Root::export_all_to(&dir).unwrap();
    Root::export_all_to(&dir).unwrap();
    assert!(dir.join("Root.py").is_file());
    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn checks_each_generic_instantiation_before_writing() {
    let result = MixedSets::export_to_string();
    assert!(
        matches!(&result, Err(ExportError::UnhashableType(name)) if name == "Item"),
        "{result:?}"
    );
}

#[test]
fn supports_as_concrete_and_explicit_bounds() {
    let concrete = ConcretePage::<NotPython, Item>::export_to_string().unwrap();
    assert!(
        concrete.contains("class ConcretePage(Generic[U]):"),
        "{concrete}"
    );
    assert!(concrete.contains("fixed: str"), "{concrete}");
    assert!(concrete.contains("dynamic: U"), "{concrete}");
    assert!(!concrete.contains("T = TypeVar"), "{concrete}");
    assert!(concrete.contains("U = TypeVar(\"U\")"), "{concrete}");
    assert_eq!(
        ConcretePage::<NotPython, Item>::inline(),
        "ConcretePage[Item]"
    );

    assert!(ExplicitBound::<Item>::export_to_string()
        .unwrap()
        .contains("value: T"));
    assert!(FieldAs::export_to_string()
        .unwrap()
        .contains("numeric_id: str"));
    assert_eq!(TransparentId::inline(), "str");
    assert_eq!(GenericTransparent::<String>::inline(), "str");
    assert_eq!(
        GenericTransparent::<String>::type_spec_with(&[rust_py_models::TypeSpec::named("T")])
            .annotation(),
        "T"
    );
    assert!(TransparentId::model_spec().unwrap().is_none());
}

#[test]
fn explicit_newtype_preserves_python_type_identity() {
    let user_id = UserId::export_to_string().unwrap();
    assert!(user_id.contains("from typing import NewType"), "{user_id}");
    assert!(
        user_id.contains("UserId = NewType(\"UserId\", int)"),
        "{user_id}"
    );
    let holder = NewTypeHolder::export_to_string().unwrap();
    assert!(holder.contains("user_id: UserId"), "{holder}");
    assert!(holder.contains("from .UserId import UserId"), "{holder}");
}

#[test]
fn recursive_and_defaulted_generics_preserve_arguments() {
    let node = GenericNode::<Item>::export_to_string().unwrap();
    assert!(node.contains("class GenericNode(Generic[T]):"), "{node}");
    assert!(node.contains("children: list[GenericNode[T]]"), "{node}");

    let left = GenericLeft::<Item>::export_to_string().unwrap();
    assert!(left.contains("right: GenericRight[T] | None"), "{left}");
    assert!(
        left.contains("from .GenericRight import GenericRight  # noqa: E402 - cyclic dependency"),
        "{left}"
    );

    let root = GenericEdgeRoot::export_to_string().unwrap();
    assert!(root.contains("node: GenericNode[Item]"), "{root}");
    assert!(root.contains("left: GenericLeft[Item]"), "{root}");
    assert!(root.contains("default_page: DefaultPage[str]"), "{root}");
}
