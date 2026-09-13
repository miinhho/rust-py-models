use crate::{Dependency, ExportError, PY};
use std::borrow::Cow;

impl<T: PY> PY for Option<T> {
    fn name() -> String {
        Self::inline()
    }
    fn inline() -> String {
        format!("{} | None", T::inline())
    }
    fn inline_with(args: &[String]) -> String {
        format!("{} | None", args[0])
    }
    fn referenced_types() -> Vec<Dependency> {
        T::referenced_types()
    }
    fn annotation_imports() -> Vec<String> {
        T::annotation_imports()
    }
    fn local_imports() -> Vec<String> {
        Vec::new()
    }
    fn is_hashable() -> bool {
        T::is_hashable()
    }
    fn annotation_validate() -> Result<(), ExportError> {
        T::annotation_validate()
    }
}

macro_rules! transparent {
    ($($ty:ty),* $(,)?) => {$ (
        impl<T: PY> PY for $ty {
            fn name() -> String { T::name() }
            fn inline() -> String { T::inline() }
            fn inline_with(args: &[String]) -> String { args[0].clone() }
            fn referenced_types() -> Vec<Dependency> { T::referenced_types() }
            fn annotation_imports() -> Vec<String> { T::annotation_imports() }
            fn local_imports() -> Vec<String> { Vec::new() }
            fn is_hashable() -> bool { T::is_hashable() }
            fn annotation_validate() -> Result<(), ExportError> { T::annotation_validate() }
        }
    )*};
}

transparent!(Box<T>, std::rc::Rc<T>, std::sync::Arc<T>);

impl<T: PY> PY for &T {
    fn name() -> String {
        T::name()
    }
    fn inline() -> String {
        T::inline()
    }
    fn inline_with(args: &[String]) -> String {
        args[0].clone()
    }
    fn referenced_types() -> Vec<Dependency> {
        T::referenced_types()
    }
    fn annotation_imports() -> Vec<String> {
        T::annotation_imports()
    }
    fn local_imports() -> Vec<String> {
        Vec::new()
    }
    fn is_hashable() -> bool {
        T::is_hashable()
    }
    fn annotation_validate() -> Result<(), ExportError> {
        T::annotation_validate()
    }
}

impl<'a, T> PY for Cow<'a, T>
where
    T: ToOwned + PY + ?Sized,
{
    fn name() -> String {
        T::name()
    }
    fn inline() -> String {
        T::inline()
    }
    fn inline_with(args: &[String]) -> String {
        args[0].clone()
    }
    fn referenced_types() -> Vec<Dependency> {
        T::referenced_types()
    }
    fn annotation_imports() -> Vec<String> {
        T::annotation_imports()
    }
    fn local_imports() -> Vec<String> {
        Vec::new()
    }
    fn is_hashable() -> bool {
        T::is_hashable()
    }
    fn annotation_validate() -> Result<(), ExportError> {
        T::annotation_validate()
    }
}
