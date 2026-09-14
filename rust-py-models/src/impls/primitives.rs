use crate::{TypeSpec, PY};
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr, SocketAddrV4, SocketAddrV6};
use std::num::{
    NonZeroI128, NonZeroI16, NonZeroI32, NonZeroI64, NonZeroI8, NonZeroIsize, NonZeroU128,
    NonZeroU16, NonZeroU32, NonZeroU64, NonZeroU8, NonZeroUsize,
};
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

macro_rules! primitive {
    ($($ty:ty => $python:expr),* $(,)?) => {$ (
        impl PY for $ty {
            fn type_spec() -> TypeSpec {
                TypeSpec::named($python).with_hashable(true)
            }
        }
    )*};
}

primitive! {
    bool => "bool", char => "str", String => "str", str => "str",
    i8 => "int", i16 => "int", i32 => "int", i64 => "int", i128 => "int", isize => "int",
    u8 => "int", u16 => "int", u32 => "int", u64 => "int", u128 => "int", usize => "int",
    NonZeroI8 => "int", NonZeroI16 => "int", NonZeroI32 => "int", NonZeroI64 => "int",
    NonZeroI128 => "int", NonZeroIsize => "int", NonZeroU8 => "int", NonZeroU16 => "int",
    NonZeroU32 => "int", NonZeroU64 => "int", NonZeroU128 => "int", NonZeroUsize => "int",
    f32 => "float", f64 => "float", () => "None",
    SocketAddr => "str", SocketAddrV4 => "str", SocketAddrV6 => "str",
}

macro_rules! qualified {
    ($($ty:ty => ($module:expr, $python:expr)),* $(,)?) => {$ (
        impl PY for $ty {
            fn type_spec() -> TypeSpec {
                TypeSpec::qualified($module, $python).with_hashable(true)
            }
        }
    )*};
}

qualified! {
    Path => ("pathlib", "Path"),
    PathBuf => ("pathlib", "Path"),
    Ipv4Addr => ("ipaddress", "IPv4Address"),
    Ipv6Addr => ("ipaddress", "IPv6Address"),
}

impl PY for Duration {
    fn type_spec() -> TypeSpec {
        TypeSpec::qualified("datetime", "timedelta").with_hashable(true)
    }
}

impl PY for SystemTime {
    fn type_spec() -> TypeSpec {
        TypeSpec::qualified("datetime", "datetime").with_hashable(true)
    }
}

impl PY for IpAddr {
    fn type_spec() -> TypeSpec {
        TypeSpec::union(vec![
            TypeSpec::qualified("ipaddress", "IPv4Address").with_hashable(true),
            TypeSpec::qualified("ipaddress", "IPv6Address").with_hashable(true),
        ])
    }
}

macro_rules! borrowed_unsized {
    ($($ty:ty),* $(,)?) => {$ (
        impl PY for &$ty {
            fn type_spec() -> TypeSpec { <$ty as PY>::type_spec() }
            fn type_spec_with(args: &[TypeSpec]) -> TypeSpec {
                <$ty as PY>::type_spec_with(args)
            }
        }
    )*};
}

borrowed_unsized!(str, Path);
