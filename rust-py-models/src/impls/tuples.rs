use crate::{TypeSpec, PY};

macro_rules! tuple_impl {
    ($($name:ident),+ $(,)?) => {
        impl<$($name: PY),+> PY for ($($name,)+) {
            fn type_spec() -> TypeSpec {
                TypeSpec::tuple(vec![$($name::type_spec()),+])
            }

            fn type_spec_with(args: &[TypeSpec]) -> TypeSpec {
                TypeSpec::tuple(args.to_vec())
            }
        }
    };
}

tuple_impl!(A);
tuple_impl!(A, B);
tuple_impl!(A, B, C);
tuple_impl!(A, B, C, D);
tuple_impl!(A, B, C, D, E);
tuple_impl!(A, B, C, D, E, F);
tuple_impl!(A, B, C, D, E, F, G);
tuple_impl!(A, B, C, D, E, F, G, H);
tuple_impl!(A, B, C, D, E, F, G, H, I);
tuple_impl!(A, B, C, D, E, F, G, H, I, J);
