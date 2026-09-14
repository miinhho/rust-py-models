use crate::{TypeSpec, PY};

macro_rules! version_string {
    ($($ty:ty),* $(,)?) => {$ (
        impl PY for $ty {
            fn type_spec() -> TypeSpec {
                TypeSpec::named("str").with_hashable(true)
            }
        }
    )*};
}

version_string!(::semver::Version, ::semver::VersionReq);
