use crate::{TypeSpec, PY};

macro_rules! ordered {
    ($($ty:ty),* $(,)?) => {$ (
        impl<T: PY> PY for $ty {
            fn type_spec() -> TypeSpec {
                T::type_spec().with_hashable(true)
            }

            fn type_spec_with(args: &[TypeSpec]) -> TypeSpec {
                args[0].clone().with_hashable(true)
            }
        }
    )*};
}

ordered!(::ordered_float::OrderedFloat<T>, ::ordered_float::NotNan<T>);
