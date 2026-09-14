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

struct HiddenError;

#[derive(PY)]
struct ResultPage<T, E> {
    value: Result<T, E>,
    history: Vec<std::result::Result<T, E>>,
}

#[derive(PY)]
#[py(as = "Result<T, E>")]
struct ResultAlias<E, T>(E, T);

#[derive(PY)]
struct ResultWithError<T, E> {
    value: Result<T, E>,
    detail: E,
}

#[derive(PY)]
enum ResultChoice<T, E> {
    Ready(Result<T, E>),
    Empty,
}

#[derive(PY)]
#[py(export)]
struct ResultRoot {
    page: ResultPage<Item, HiddenError>,
}

#[derive(PY)]
struct PlainMapHolder<K, V, S> {
    map: std::collections::HashMap<K, V, S>,
}

#[cfg(feature = "indexmap-impl")]
#[derive(PY)]
struct IndexedMapHolder<K, V, S> {
    map: indexmap::IndexMap<K, V, S>,
}

#[cfg(feature = "chrono-impl")]
#[derive(PY)]
struct DateTimeHolder<Tz: chrono::TimeZone> {
    value: chrono::DateTime<Tz>,
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

#[rust_py_models::py_models]
mod projected_graph {
    use super::{HiddenError, Item};
    use rust_py_models::PY;
    use std::collections::HashMap;

    #[derive(PY)]
    pub(super) struct Inner<T, E> {
        value: Result<T, E>,
    }

    #[derive(PY)]
    pub(super) struct Outer<T, E> {
        inner: Inner<T, E>,
    }

    #[derive(PY)]
    pub(super) struct ProjectedLeft<T, E> {
        right: Option<Box<ProjectedRight<T, E>>>,
    }

    #[derive(PY)]
    pub(super) struct ProjectedRight<T, E> {
        left: Option<Box<ProjectedLeft<T, E>>>,
        value: Result<T, E>,
    }

    #[derive(PY)]
    pub(super) struct MapHolder<K, V, S> {
        map: HashMap<K, V, S>,
    }

    #[derive(PY)]
    pub(super) struct VisibleLeft<T, E> {
        right: Option<Box<VisibleRight<T, E>>>,
    }

    #[derive(PY)]
    pub(super) struct VisibleRight<T, E> {
        left: Option<Box<VisibleLeft<T, E>>>,
        value: Result<T, E>,
        error: E,
    }

    #[derive(PY)]
    #[py(export)]
    pub(super) struct GroupRoot {
        outer: Outer<Item, HiddenError>,
    }

    pub(super) fn verify() {
        let outer = Outer::<Item, HiddenError>::export_to_string().unwrap();
        assert!(outer.contains("class Outer(Generic[T]):"), "{outer}");
        assert!(outer.contains("inner: Inner[T]"), "{outer}");
        assert_eq!(Outer::<Item, HiddenError>::inline(), "Outer[Item]");

        let left = ProjectedLeft::<Item, HiddenError>::export_to_string().unwrap();
        assert!(left.contains("class ProjectedLeft(Generic[T]):"), "{left}");
        assert!(left.contains("right: ProjectedRight[T] | None"), "{left}");
        assert_eq!(
            ProjectedLeft::<Item, HiddenError>::inline(),
            "ProjectedLeft[Item]"
        );

        let map = MapHolder::<u64, String, HiddenError>::export_to_string().unwrap();
        assert!(map.contains("class MapHolder(Generic[K, V]):"), "{map}");
        assert!(map.contains("map: dict[K, V]"), "{map}");

        let visible = VisibleLeft::<Item, String>::export_to_string().unwrap();
        assert!(
            visible.contains("class VisibleLeft(Generic[T, E]):"),
            "{visible}"
        );
        assert!(
            visible.contains("right: VisibleRight[T, E] | None"),
            "{visible}"
        );
    }
}

#[derive(PY)]
#[py(export)]
struct ProjectedRoot {
    outer: projected_graph::Outer<Item, HiddenError>,
    left: projected_graph::ProjectedLeft<Item, HiddenError>,
    map: projected_graph::MapHolder<u64, String, HiddenError>,
}

#[test]
fn grouped_models_project_indirect_and_cyclic_parameters() {
    projected_graph::verify();
    let map = PlainMapHolder::<u64, String, HiddenError>::export_to_string().unwrap();
    assert!(
        map.contains("class PlainMapHolder(Generic[K, V]):"),
        "{map}"
    );
    assert!(map.contains("map: dict[K, V]"), "{map}");

    #[cfg(feature = "indexmap-impl")]
    {
        let indexed = IndexedMapHolder::<u64, String, HiddenError>::export_to_string().unwrap();
        assert!(
            indexed.contains("class IndexedMapHolder(Generic[K, V]):"),
            "{indexed}"
        );
        assert!(indexed.contains("map: dict[K, V]"), "{indexed}");
    }

    #[cfg(feature = "chrono-impl")]
    {
        let datetime = DateTimeHolder::<chrono::Utc>::export_to_string().unwrap();
        assert!(datetime.contains("class DateTimeHolder:"), "{datetime}");
        assert!(datetime.contains("value: datetime.datetime"), "{datetime}");
    }
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
    assert_eq!(
        ConcretePage::<NotPython, Item>::type_spec_with(&[
            rust_py_models::TypeSpec::named("ignored"),
            rust_py_models::TypeSpec::named("Item"),
        ])
        .annotation(),
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
fn result_error_generic_is_absent_from_python_model() {
    assert_eq!(
        ResultPage::<Item, HiddenError>::inline(),
        "ResultPage[Item]"
    );
    assert_eq!(
        ResultPage::<Item, HiddenError>::type_spec_with(&[
            rust_py_models::TypeSpec::named("Item"),
            rust_py_models::TypeSpec::named("ignored"),
        ])
        .annotation(),
        "ResultPage[Item]"
    );
    let page = ResultPage::<Item, HiddenError>::export_to_string().unwrap();
    assert!(page.contains("class ResultPage(Generic[T]):"), "{page}");
    assert!(page.contains("value: T"), "{page}");
    assert!(page.contains("history: list[T]"), "{page}");
    let root = ResultRoot::export_to_string().unwrap();
    assert!(root.contains("page: ResultPage[Item]"), "{root}");
    assert_eq!(ResultAlias::<HiddenError, Item>::inline(), "Item");
    assert_eq!(
        ResultAlias::<HiddenError, Item>::type_spec_with(&[
            rust_py_models::TypeSpec::named("wrong"),
            rust_py_models::TypeSpec::named("right"),
        ])
        .annotation(),
        "right"
    );
    let visible_error = ResultWithError::<Item, String>::export_to_string().unwrap();
    assert!(
        visible_error.contains("class ResultWithError(Generic[T, E]):"),
        "{visible_error}"
    );
    let choice = ResultChoice::<Item, HiddenError>::export_to_string().unwrap();
    assert!(
        choice.contains("class ResultChoiceReady(Generic[T]):"),
        "{choice}"
    );
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
