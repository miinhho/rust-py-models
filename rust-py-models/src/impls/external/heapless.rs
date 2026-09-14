use crate::{TypeSpec, PY};

macro_rules! list_like {
    ($($ty:ty),* $(,)?) => {$ (
        impl<T: PY, const N: usize> PY for $ty {
            fn type_spec() -> TypeSpec {
                TypeSpec::list(T::type_spec())
            }

            fn type_spec_with(args: &[TypeSpec]) -> TypeSpec {
                TypeSpec::list(args[0].clone())
            }
        }
    )*};
}

list_like!(::heapless::Vec<T, N>, ::heapless::Deque<T, N>);

impl<const N: usize> PY for ::heapless::String<N> {
    fn type_spec() -> TypeSpec {
        TypeSpec::named("str").with_hashable(true)
    }
}
