use crate::PY;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use std::num::{
    NonZeroI128, NonZeroI16, NonZeroI32, NonZeroI64, NonZeroI8, NonZeroIsize, NonZeroU128,
    NonZeroU16, NonZeroU32, NonZeroU64, NonZeroU8, NonZeroUsize,
};
use std::path::{Path, PathBuf};

macro_rules! primitive {
    ($($ty:ty => $python:expr),* $(,)?) => {$ (
        impl PY for $ty {
            fn name() -> String { $python.into() }
            fn inline() -> String { $python.into() }
            fn is_hashable() -> bool { true }
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
}

macro_rules! native {
    ($($ty:ty => ($python:expr, $import:expr)),* $(,)?) => {$ (
        impl PY for $ty {
            fn name() -> String { $python.into() }
            fn inline() -> String { $python.into() }
            fn annotation_imports() -> Vec<String> { vec![$import.into()] }
            fn is_hashable() -> bool { true }
        }
    )*};
}

native! {
    Path => ("pathlib.Path", "import pathlib"),
    PathBuf => ("pathlib.Path", "import pathlib"),
    IpAddr => ("ipaddress.IPv4Address | ipaddress.IPv6Address", "import ipaddress"),
    Ipv4Addr => ("ipaddress.IPv4Address", "import ipaddress"),
    Ipv6Addr => ("ipaddress.IPv6Address", "import ipaddress"),
}

macro_rules! borrowed_unsized {
    ($($ty:ty),* $(,)?) => {$ (
        impl<'a> PY for &'a $ty {
            fn name() -> String { <$ty as PY>::name() }
            fn inline() -> String { <$ty as PY>::inline() }
            fn annotation_imports() -> Vec<String> { <$ty as PY>::annotation_imports() }
            fn is_hashable() -> bool { <$ty as PY>::is_hashable() }
        }
    )*};
}

borrowed_unsized!(str, Path);
