use crate::{TypeSpec, PY};
use std::borrow::Cow;

impl<T: PY> PY for Option<T> {
    fn type_spec() -> TypeSpec {
        TypeSpec::optional(T::type_spec())
    }

    fn type_spec_with(args: &[TypeSpec]) -> TypeSpec {
        TypeSpec::optional(args[0].clone())
    }
}

macro_rules! transparent {
    ($($ty:ty),* $(,)?) => {$ (
        impl<T: PY> PY for $ty {
            fn type_spec() -> TypeSpec { T::type_spec() }
            fn type_spec_with(args: &[TypeSpec]) -> TypeSpec { args[0].clone() }
        }
    )*};
}

transparent!(
    Box<T>,
    std::cell::Cell<T>,
    std::cell::RefCell<T>,
    std::rc::Rc<T>,
    std::sync::Arc<T>,
    std::sync::Mutex<T>,
    std::sync::RwLock<T>
);

impl<T: PY> PY for std::rc::Weak<T> {
    fn type_spec() -> TypeSpec {
        TypeSpec::optional(T::type_spec())
    }

    fn type_spec_with(args: &[TypeSpec]) -> TypeSpec {
        TypeSpec::optional(args[0].clone())
    }
}

impl<T: PY> PY for std::sync::Weak<T> {
    fn type_spec() -> TypeSpec {
        TypeSpec::optional(T::type_spec())
    }

    fn type_spec_with(args: &[TypeSpec]) -> TypeSpec {
        TypeSpec::optional(args[0].clone())
    }
}

impl<T: PY> PY for &T {
    fn type_spec() -> TypeSpec {
        T::type_spec()
    }

    fn type_spec_with(args: &[TypeSpec]) -> TypeSpec {
        args[0].clone()
    }
}

impl<T> PY for Cow<'_, T>
where
    T: ToOwned + PY + ?Sized,
{
    fn type_spec() -> TypeSpec {
        T::type_spec()
    }

    fn type_spec_with(args: &[TypeSpec]) -> TypeSpec {
        args[0].clone()
    }
}
