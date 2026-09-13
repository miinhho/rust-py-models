use crate::{Dependency, ExportError, PY};

macro_rules! tuple_impl {
    ($($name:ident),+ $(,)?) => {
        impl<$($name: PY),+> PY for ($($name,)+) {
            fn name() -> String { Self::inline() }
            fn inline() -> String {
                format!("tuple[{}]", [$($name::inline()),+].join(", "))
            }
            fn inline_with(args: &[String]) -> String { format!("tuple[{}]", args.join(", ")) }
            fn referenced_types() -> Vec<Dependency> {
                let mut deps = Vec::new();
                $(deps.extend($name::referenced_types());)+
                deps
            }
            fn annotation_imports() -> Vec<String> {
                let mut imports = Vec::new();
                $(imports.extend($name::annotation_imports());)+
                imports
            }
            fn local_imports() -> Vec<String> { Vec::new() }
            fn is_hashable() -> bool { true $(&& $name::is_hashable())+ }
            fn annotation_validate() -> Result<(), ExportError> {
                $($name::annotation_validate()?;)+
                Ok(())
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
