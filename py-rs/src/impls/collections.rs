use crate::{Dependency, ExportError, PY};
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet, LinkedList, VecDeque};

impl<T: PY> PY for Vec<T> {
    fn name() -> String {
        Self::inline()
    }
    fn inline() -> String {
        format!("list[{}]", T::inline())
    }
    fn inline_with(args: &[String]) -> String {
        format!("list[{}]", args[0])
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
    fn annotation_validate() -> Result<(), ExportError> {
        T::annotation_validate()
    }
}

macro_rules! list_like {
    ($($ty:ty),* $(,)?) => {$ (
        impl<T: PY> PY for $ty {
            fn name() -> String { <Vec<T> as PY>::name() }
            fn inline() -> String { <Vec<T> as PY>::inline() }
            fn inline_with(args: &[String]) -> String { <Vec<T> as PY>::inline_with(args) }
            fn referenced_types() -> Vec<Dependency> { T::referenced_types() }
            fn annotation_imports() -> Vec<String> { T::annotation_imports() }
            fn local_imports() -> Vec<String> { Vec::new() }
            fn annotation_validate() -> Result<(), ExportError> { T::annotation_validate() }
        }
    )*};
}

list_like!(LinkedList<T>);

impl<T: PY> PY for [T] {
    fn name() -> String {
        <Vec<T> as PY>::name()
    }
    fn inline() -> String {
        <Vec<T> as PY>::inline()
    }
    fn inline_with(args: &[String]) -> String {
        <Vec<T> as PY>::inline_with(args)
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
    fn annotation_validate() -> Result<(), ExportError> {
        T::annotation_validate()
    }
}

impl<T: PY> PY for VecDeque<T> {
    fn name() -> String {
        Self::inline()
    }
    fn inline() -> String {
        format!("collections.deque[{}]", T::inline())
    }
    fn inline_with(args: &[String]) -> String {
        format!("collections.deque[{}]", args[0])
    }
    fn referenced_types() -> Vec<Dependency> {
        T::referenced_types()
    }
    fn annotation_imports() -> Vec<String> {
        let mut imports = vec!["import collections".into()];
        imports.extend(T::annotation_imports());
        imports
    }
    fn local_imports() -> Vec<String> {
        vec!["import collections".into()]
    }
    fn annotation_validate() -> Result<(), ExportError> {
        T::annotation_validate()
    }
}

impl<T: PY> PY for &[T] {
    fn name() -> String {
        <Vec<T> as PY>::name()
    }
    fn inline() -> String {
        <Vec<T> as PY>::inline()
    }
    fn inline_with(args: &[String]) -> String {
        <Vec<T> as PY>::inline_with(args)
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
    fn annotation_validate() -> Result<(), ExportError> {
        T::annotation_validate()
    }
}

impl<T: PY, const N: usize> PY for [T; N] {
    fn name() -> String {
        Self::inline()
    }
    fn inline() -> String {
        match N {
            0 => "tuple[()]".into(),
            1..=12 => format!("tuple[{}]", vec![T::inline(); N].join(", ")),
            _ => format!("tuple[{}, ...]", T::inline()),
        }
    }
    fn inline_with(args: &[String]) -> String {
        match N {
            0 => "tuple[()]".into(),
            1..=12 => format!("tuple[{}]", vec![args[0].clone(); N].join(", ")),
            _ => format!("tuple[{}, ...]", args[0]),
        }
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
        N == 0 || T::is_hashable()
    }
    fn annotation_validate() -> Result<(), ExportError> {
        T::annotation_validate()
    }
}

macro_rules! set_like {
    ($($ty:ty),* $(,)?) => {$ (
        impl<T: PY> PY for $ty {
            fn name() -> String { Self::inline() }
            fn inline() -> String { format!("set[{}]", T::inline()) }
            fn inline_with(args: &[String]) -> String { format!("set[{}]", args[0]) }
            fn referenced_types() -> Vec<Dependency> { T::referenced_types() }
            fn annotation_imports() -> Vec<String> { T::annotation_imports() }
            fn local_imports() -> Vec<String> { Vec::new() }
            fn annotation_validate() -> Result<(), ExportError> {
                T::annotation_validate()?;
                if T::is_hashable() { Ok(()) } else {
                    Err(ExportError::UnhashableType(T::name()))
                }
            }
        }
    )*};
}

set_like!(BTreeSet<T>);

impl<T: PY, S> PY for HashSet<T, S> {
    fn name() -> String {
        Self::inline()
    }
    fn inline() -> String {
        format!("set[{}]", T::inline())
    }
    fn inline_with(args: &[String]) -> String {
        format!("set[{}]", args[0])
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
    fn annotation_validate() -> Result<(), ExportError> {
        T::annotation_validate()?;
        if T::is_hashable() {
            Ok(())
        } else {
            Err(ExportError::UnhashableType(T::name()))
        }
    }
}

macro_rules! map_like {
    ($($ty:ty),* $(,)?) => {$ (
        impl<K: PY, V: PY> PY for $ty {
            fn name() -> String { Self::inline() }
            fn inline() -> String { format!("dict[{}, {}]", K::inline(), V::inline()) }
            fn inline_with(args: &[String]) -> String { format!("dict[{}, {}]", args[0], args[1]) }
            fn referenced_types() -> Vec<Dependency> {
                let mut deps = K::referenced_types();
                deps.extend(V::referenced_types());
                deps
            }
            fn annotation_imports() -> Vec<String> {
                let mut imports = K::annotation_imports();
                imports.extend(V::annotation_imports());
                imports
            }
            fn local_imports() -> Vec<String> { Vec::new() }
            fn annotation_validate() -> Result<(), ExportError> {
                K::annotation_validate()?;
                V::annotation_validate()?;
                if K::is_hashable() { Ok(()) } else {
                    Err(ExportError::UnhashableType(K::name()))
                }
            }
        }
    )*};
}

map_like!(BTreeMap<K, V>);

impl<K: PY, V: PY, S> PY for HashMap<K, V, S> {
    fn name() -> String {
        Self::inline()
    }
    fn inline() -> String {
        format!("dict[{}, {}]", K::inline(), V::inline())
    }
    fn inline_with(args: &[String]) -> String {
        format!("dict[{}, {}]", args[0], args[1])
    }
    fn referenced_types() -> Vec<Dependency> {
        let mut deps = K::referenced_types();
        deps.extend(V::referenced_types());
        deps
    }
    fn annotation_imports() -> Vec<String> {
        let mut imports = K::annotation_imports();
        imports.extend(V::annotation_imports());
        imports
    }
    fn local_imports() -> Vec<String> {
        Vec::new()
    }
    fn annotation_validate() -> Result<(), ExportError> {
        K::annotation_validate()?;
        V::annotation_validate()?;
        if K::is_hashable() {
            Ok(())
        } else {
            Err(ExportError::UnhashableType(K::name()))
        }
    }
}
