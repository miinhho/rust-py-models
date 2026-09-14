use crate::{TypeSpec, PY};

macro_rules! transparent {
    ($($ty:ty),* $(,)?) => {$ (
        impl<T: PY> PY for $ty {
            fn type_spec() -> TypeSpec {
                T::type_spec()
            }

            fn type_spec_with(args: &[TypeSpec]) -> TypeSpec {
                args[0].clone()
            }
        }
    )*};
}

transparent!(::tokio::sync::Mutex<T>, ::tokio::sync::RwLock<T>);

impl<T: PY> PY for ::tokio::sync::OnceCell<T> {
    fn type_spec() -> TypeSpec {
        TypeSpec::optional(T::type_spec())
    }

    fn type_spec_with(args: &[TypeSpec]) -> TypeSpec {
        TypeSpec::optional(args[0].clone())
    }
}
