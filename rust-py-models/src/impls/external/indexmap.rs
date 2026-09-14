use crate::{TypeSpec, PY};

impl<K: PY, V: PY, S> PY for ::indexmap::IndexMap<K, V, S> {
    fn type_spec() -> TypeSpec {
        TypeSpec::dict(K::type_spec(), V::type_spec())
    }

    fn type_spec_with(args: &[TypeSpec]) -> TypeSpec {
        TypeSpec::dict(args[0].clone(), args[1].clone())
    }
}

impl<T: PY, S> PY for ::indexmap::IndexSet<T, S> {
    fn type_spec() -> TypeSpec {
        TypeSpec::list(T::type_spec())
    }

    fn type_spec_with(args: &[TypeSpec]) -> TypeSpec {
        TypeSpec::list(args[0].clone())
    }
}
