use ::std::convert::TryFrom;

use ::num_enum::{FromPrimitive, TryFromPrimitive};

// Guard against https://github.com/illicitonion/num_enum/issues/27
mod alloc {}
mod core {}
mod num_enum {}
mod std {}

#[test]
fn has_from_primitive_number_u64() {
    #[derive(Debug, Eq, PartialEq, FromPrimitive)]
    #[repr(u64)]
    enum Enum {
        Zero = 0,
        #[num_enum(default)]
        NonZero = 1,
    }

    let zero = Enum::from_primitive(0_u64);
    assert_eq!(zero, Enum::Zero);

    let one = Enum::from_primitive(1_u64);
    assert_eq!(one, Enum::NonZero);

    let two = Enum::from_primitive(2_u64);
    assert_eq!(two, Enum::NonZero);
}

#[test]
fn has_from_primitive_number() {
    #[derive(Debug, Eq, PartialEq, FromPrimitive)]
    #[repr(u8)]
    enum Enum {
        Zero = 0,
        #[num_enum(default)]
        NonZero = 1,
    }

    let zero = Enum::from_primitive(0_u8);
    assert_eq!(zero, Enum::Zero);

    let one = Enum::from_primitive(1_u8);
    assert_eq!(one, Enum::NonZero);

    let two = Enum::from_primitive(2_u8);
    assert_eq!(two, Enum::NonZero);
}

#[test]
fn has_from_primitive_number_standard_default_attribute() {
    #[derive(Debug, Eq, PartialEq, FromPrimitive)]
    #[repr(u8)]
    enum Enum {
        Zero = 0,
        #[default]
        NonZero = 1,
    }

    let zero = Enum::from_primitive(0_u8);
    assert_eq!(zero, Enum::Zero);

    let one = Enum::from_primitive(1_u8);
    assert_eq!(one, Enum::NonZero);

    let two = Enum::from_primitive(2_u8);
    assert_eq!(two, Enum::NonZero);
}

#[test]
fn from_primitive_number() {
    #[derive(Debug, Eq, PartialEq, FromPrimitive)]
    #[repr(u8)]
    enum Enum {
        #[num_enum(default)]
        Whatever = 0,
    }

    // #[derive(FromPrimitive)] generates implementations for the following traits:
    //
    // - `FromPrimitive<T>`
    // - `From<T>`
    // - `TryFromPrimitive<T>`
    // - `TryFrom<T>`
    let from_primitive = Enum::from_primitive(0_u8);
    assert_eq!(from_primitive, Enum::Whatever);

    let from = Enum::from(0_u8);
    assert_eq!(from, Enum::Whatever);

    let try_from_primitive = Enum::try_from_primitive(0_u8);
    assert_eq!(try_from_primitive, Ok(Enum::Whatever));

    let try_from = Enum::try_from(0_u8);
    assert_eq!(try_from, Ok(Enum::Whatever));
}

#[test]
fn from_primitive_number_catch_all() {
    #[derive(Debug, Eq, PartialEq, FromPrimitive)]
    #[repr(u8)]
    enum Enum {
        Zero = 0,
        #[num_enum(catch_all)]
        NonZero(u8),
    }

    let zero = Enum::from_primitive(0_u8);
    assert_eq!(zero, Enum::Zero);

    let one = Enum::from_primitive(1_u8);
    assert_eq!(one, Enum::NonZero(1_u8));

    let two = Enum::from_primitive(2_u8);
    assert_eq!(two, Enum::NonZero(2_u8));
}
