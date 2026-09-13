#![allow(dead_code)]

use py_rs::ExportError;
use py_rs::PY;
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
    assert_eq!(
        Page::<PathBuf>::declaration_id(),
        Page::<Item>::declaration_id()
    );
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
    assert!(Borrowed::<'static>::decl().contains("label: str"));
    let pair = Pair::<PathBuf, Item>::export_to_string().unwrap();
    assert!(pair.contains("class Pair(Generic[A, B]):"), "{pair}");
    assert!(pair.contains("left: A"), "{pair}");
    assert!(pair.contains("right: B"), "{pair}");
}

#[test]
fn exports_generic_graph() {
    Root::export_all().unwrap();
    Root::export_all().unwrap();
}

#[test]
fn checks_each_generic_instantiation_before_writing() {
    assert!(
        matches!(MixedSets::export_all(), Err(ExportError::UnhashableType(name)) if name == "Item")
    );
}
