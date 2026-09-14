use crate::{TypeSpec, PY};
use std::collections::{BTreeMap, BTreeSet, BinaryHeap, HashMap, HashSet, LinkedList, VecDeque};

impl<T: PY> PY for Vec<T> {
    fn type_spec() -> TypeSpec {
        TypeSpec::list(T::type_spec())
    }

    fn type_spec_with(args: &[TypeSpec]) -> TypeSpec {
        TypeSpec::list(args[0].clone())
    }
}

macro_rules! list_like {
    ($($ty:ty),* $(,)?) => {$ (
        impl<T: PY> PY for $ty {
            fn type_spec() -> TypeSpec { TypeSpec::list(T::type_spec()) }
            fn type_spec_with(args: &[TypeSpec]) -> TypeSpec {
                TypeSpec::list(args[0].clone())
            }
        }
    )*};
}

list_like!(BinaryHeap<T>, LinkedList<T>);

impl<T: PY> PY for [T] {
    fn type_spec() -> TypeSpec {
        TypeSpec::list(T::type_spec())
    }

    fn type_spec_with(args: &[TypeSpec]) -> TypeSpec {
        TypeSpec::list(args[0].clone())
    }
}

impl<T: PY> PY for VecDeque<T> {
    fn type_spec() -> TypeSpec {
        TypeSpec::deque(T::type_spec())
    }

    fn type_spec_with(args: &[TypeSpec]) -> TypeSpec {
        TypeSpec::deque(args[0].clone())
    }
}

impl<T: PY> PY for &[T] {
    fn type_spec() -> TypeSpec {
        TypeSpec::list(T::type_spec())
    }

    fn type_spec_with(args: &[TypeSpec]) -> TypeSpec {
        TypeSpec::list(args[0].clone())
    }
}

impl<T: PY, const N: usize> PY for [T; N] {
    fn type_spec() -> TypeSpec {
        array_spec::<N>(T::type_spec())
    }

    fn type_spec_with(args: &[TypeSpec]) -> TypeSpec {
        array_spec::<N>(args[0].clone())
    }
}

fn array_spec<const N: usize>(item: TypeSpec) -> TypeSpec {
    match N {
        0 => TypeSpec::tuple(Vec::new()),
        1..=12 => TypeSpec::tuple(vec![item; N]),
        _ => TypeSpec::variadic_tuple(item),
    }
}

macro_rules! set_like {
    ($($ty:ty),* $(,)?) => {$ (
        impl<T: PY> PY for $ty {
            fn type_spec() -> TypeSpec { TypeSpec::set(T::type_spec()) }
            fn type_spec_with(args: &[TypeSpec]) -> TypeSpec {
                TypeSpec::set(args[0].clone())
            }
        }
    )*};
}

set_like!(BTreeSet<T>);

impl<T: PY, S> PY for HashSet<T, S> {
    fn type_spec() -> TypeSpec {
        TypeSpec::set(T::type_spec())
    }

    fn type_spec_with(args: &[TypeSpec]) -> TypeSpec {
        TypeSpec::set(args[0].clone())
    }
}

macro_rules! map_like {
    ($($ty:ty),* $(,)?) => {$ (
        impl<K: PY, V: PY> PY for $ty {
            fn type_spec() -> TypeSpec {
                TypeSpec::dict(K::type_spec(), V::type_spec())
            }

            fn type_spec_with(args: &[TypeSpec]) -> TypeSpec {
                TypeSpec::dict(args[0].clone(), args[1].clone())
            }
        }
    )*};
}

map_like!(BTreeMap<K, V>);

impl<K: PY, V: PY, S> PY for HashMap<K, V, S> {
    fn type_spec() -> TypeSpec {
        TypeSpec::dict(K::type_spec(), V::type_spec())
    }

    fn type_spec_with(args: &[TypeSpec]) -> TypeSpec {
        TypeSpec::dict(args[0].clone(), args[1].clone())
    }
}
