#![allow(non_upper_case_globals)]

use ::num_enum::ConstFromPrimitive;

// Guard against https://github.com/illicitonion/num_enum/issues/27
mod alloc {}
mod core {}
mod num_enum {}
mod std {}

macro_rules! has_from_primitive_number {
    ( $type:ty ) => {
        paste::paste! {
            #[test]
            fn [<has_from_primitive_number_ $type>]() {
                #[derive(Debug, Eq, PartialEq, ConstFromPrimitive)]
                #[repr($type)]
                enum Enum {
                    Zero = 0,
                    #[num_enum(default)]
                    NonZero = 1,
                }

                const zero: Enum = Enum::const_from(0 as $type);
                assert_eq!(zero, Enum::Zero);

                const one: Enum = Enum::const_from(1 as $type);
                assert_eq!(one, Enum::NonZero);

                const two: Enum = Enum::const_from(2 as $type);
                assert_eq!(two, Enum::NonZero);
            }
        }
    };
}

has_from_primitive_number!(u8);
has_from_primitive_number!(u16);
has_from_primitive_number!(u32);
has_from_primitive_number!(u64);
has_from_primitive_number!(usize);
has_from_primitive_number!(i8);
has_from_primitive_number!(i16);
has_from_primitive_number!(i32);
has_from_primitive_number!(i64);
has_from_primitive_number!(isize);
// repr with 128-bit type is unstable
// has_from_primitive_number!(u128);
// has_from_primitive_number!(i128);

#[test]
fn has_from_primitive_number_standard_default_attribute() {
    #[derive(Debug, Eq, PartialEq, ConstFromPrimitive)]
    #[repr(u8)]
    enum Enum {
        Zero = 0,
        #[default]
        NonZero = 1,
    }

    const zero: Enum = Enum::const_from(0_u8);
    assert_eq!(zero, Enum::Zero);

    const one: Enum = Enum::const_from(1_u8);
    assert_eq!(one, Enum::NonZero);

    const two: Enum = Enum::const_from(2_u8);
    assert_eq!(two, Enum::NonZero);
}

#[test]
fn from_primitive_number_catch_all() {
    #[derive(Debug, Eq, PartialEq, ConstFromPrimitive)]
    #[repr(u8)]
    enum Enum {
        Zero = 0,
        #[num_enum(catch_all)]
        NonZero(u8),
    }

    const zero: Enum = Enum::const_from(0_u8);
    assert_eq!(zero, Enum::Zero);

    const one: Enum = Enum::const_from(1_u8);
    assert_eq!(one, Enum::NonZero(1_u8));

    const two: Enum = Enum::const_from(2_u8);
    assert_eq!(two, Enum::NonZero(2_u8));
}

#[cfg(feature = "complex-expressions")]
#[test]
fn from_primitive_number_with_inclusive_range() {
    #[derive(Debug, Eq, PartialEq, ConstFromPrimitive)]
    #[repr(u8)]
    enum Enum {
        Zero = 0,
        #[num_enum(alternatives = [2..=255])]
        NonZero,
    }

    const zero: Enum = Enum::const_from(0_u8);
    assert_eq!(zero, Enum::Zero);

    const one: Enum = Enum::const_from(1_u8);
    assert_eq!(one, Enum::NonZero);

    const two: Enum = Enum::const_from(2_u8);
    assert_eq!(two, Enum::NonZero);

    const three: Enum = Enum::const_from(3_u8);
    assert_eq!(three, Enum::NonZero);

    const twofivefive: Enum = Enum::const_from(255_u8);
    assert_eq!(twofivefive, Enum::NonZero);
}
