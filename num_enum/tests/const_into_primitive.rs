#![allow(non_upper_case_globals)]

use ::num_enum::ConstIntoPrimitive;

// Guard against https://github.com/illicitonion/num_enum/issues/27
mod alloc {}
mod core {}
mod num_enum {}
mod std {}

#[derive(ConstIntoPrimitive)]
#[repr(u8)]
enum Enum {
    Zero,
    One,
    Two,
}

#[test]
fn simple() {
    const zero: u8 = Enum::Zero.const_into();
    assert_eq!(zero, 0u8);

    const one: u8 = Enum::One.const_into();
    assert_eq!(one, 1u8);

    const two: u8 = Enum::Two.const_into();
    assert_eq!(two, 2u8);
}

#[test]
fn catch_all() {
    #[derive(Debug, Eq, PartialEq, ConstIntoPrimitive)]
    #[repr(u8)]
    enum Enum {
        Zero = 0,
        #[num_enum(catch_all)]
        NonZero(u8),
    }

    const zero: u8 = Enum::Zero.const_into();
    assert_eq!(zero, 0u8);

    const one: u8 = Enum::NonZero(1u8).const_into();
    assert_eq!(one, 1u8);

    const two: u8 = Enum::NonZero(2u8).const_into();
    assert_eq!(two, 2u8);
}
