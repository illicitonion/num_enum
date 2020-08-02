#![cfg_attr(feature = "external_doc", feature(external_doc))]
#![cfg_attr(feature = "external_doc", doc(include = "../README.md"))]
#![cfg_attr(
    not(feature = "external_doc"),
    doc = "See https://docs.rs/num_enum for more info about this crate."
)]
#![cfg_attr(not(feature = "std"), no_std)]

pub use ::num_enum_derive::{FromPrimitive, IntoPrimitive, TryFromPrimitive, UnsafeFromPrimitive};

use ::core::fmt;

pub trait FromPrimitive: Sized {
    type Primitive: Copy + Eq;

    fn from_primitive(number: Self::Primitive) -> Self;
}

pub trait TryFromPrimitive: Sized {
    type Primitive: Copy + Eq + fmt::Debug;

    const NAME: &'static str;

    fn try_from_primitive(number: Self::Primitive) -> Result<Self, TryFromPrimitiveError<Self>>;
}

#[derive(::derivative::Derivative)]
#[derivative( // use derivative to remove incorrect bound on `Enum` parameter. See https://github.com/rust-lang/rust/issues/26925
    Debug(bound = ""),
    Clone(bound = ""),
    Copy(bound = ""),
    PartialEq(bound = ""),
    Eq(bound = "")
)]
pub struct TryFromPrimitiveError<Enum: TryFromPrimitive> {
    pub number: Enum::Primitive,
}

impl<Enum: TryFromPrimitive> fmt::Display for TryFromPrimitiveError<Enum> {
    fn fmt(self: &'_ Self, stream: &'_ mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            stream,
            "No discriminant in enum `{name}` matches the value `{input:?}`",
            name = Enum::NAME,
            input = self.number,
        )
    }
}

#[cfg(feature = "std")]
impl<Enum: TryFromPrimitive> ::std::error::Error for TryFromPrimitiveError<Enum> {}
