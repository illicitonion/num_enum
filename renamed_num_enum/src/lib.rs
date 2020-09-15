#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

/// Un-rename `num_enum` when in a no-std crate, since it is not supported yet.
#[cfg(not(feature = "std"))]
extern crate renamed as num_enum;

pub use submodule::test;

/// use a submodule to avoid `mod alloc {}` clashing with `::alloc`
mod submodule {
    use ::alloc::format;
    use ::core::convert::TryFrom;

    use ::renamed::{IntoPrimitive, TryFromPrimitive, UnsafeFromPrimitive};

    // Guard against https://github.com/illicitonion/num_enum/issues/27
    mod alloc {}
    mod core {}
    mod num_enum {}
    mod std {}

    #[derive(Debug, PartialEq, Eq, IntoPrimitive, TryFromPrimitive, UnsafeFromPrimitive)]
    #[repr(u8)]
    enum Number {
        Zero,
        One,
    }

    pub fn test() {
        assert_eq!(u8::from(Number::Zero), 0);

        assert_eq!(u8::from(Number::One), 1);

        assert_eq!(Number::try_from(0).unwrap(), Number::Zero);

        assert_eq!(Number::try_from(1).unwrap(), Number::One);

        assert_eq!(
            format!("{}", Number::try_from(2).unwrap_err()),
            "No discriminant in enum `Number` matches the value `2`",
        );

        #[cfg(feature = "std")]
        assert_eq!(
            Number::try_from(2).unwrap_err().to_string(),
            "No discriminant in enum `Number` matches the value `2`",
        );

        assert_eq!(unsafe { Number::from_unchecked(0) }, Number::Zero);

        assert_eq!(unsafe { Number::from_unchecked(1) }, Number::One);
    }
}
