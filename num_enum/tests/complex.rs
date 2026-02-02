use num_enum_derive::{FromPrimitive, IntoPrimitive};

// Guard against https://github.com/illicitonion/num_enum/issues/27
mod alloc {}
mod core {}
mod num_enum {}
mod std {}

#[test]
fn std_default_with_num_enum_catch_all() {
    #[derive(Debug, Eq, PartialEq, Default, FromPrimitive, IntoPrimitive)]
    #[repr(u8)]
    enum Enum {
        #[default]
        Zero = 0,
        #[num_enum(catch_all)]
        NonZero(u8),
    }

    assert_eq!(Enum::Zero, <Enum as ::core::default::Default>::default());
    assert_eq!(Enum::NonZero(5), 5u8.into());
}

#[test]
fn std_default_with_num_enum_default() {
    #[derive(Debug, Eq, PartialEq, Default, FromPrimitive, IntoPrimitive)]
    #[repr(u8)]
    enum Enum {
        #[default]
        Zero = 0,
        #[num_enum(default)]
        NonZero,
    }

    assert_eq!(Enum::Zero, <Enum as ::core::default::Default>::default());
    assert_eq!(Enum::NonZero, 5u8.into());
}
