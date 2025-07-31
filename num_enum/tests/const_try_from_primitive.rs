#![allow(non_upper_case_globals)]

use ::num_enum::{ConstTryFromPrimitive, ConstTryFromPrimitiveError};

// Guard against https://github.com/illicitonion/num_enum/issues/27
mod alloc {}
mod core {}
mod num_enum {}
mod std {}

#[test]
fn simple() {
    #[derive(Debug, Eq, PartialEq, ConstTryFromPrimitive)]
    #[repr(u8)]
    enum Enum {
        Zero,
        One,
        Two,
    }

    const zero: Result<Enum, ConstTryFromPrimitiveError<Enum>> = Enum::const_try_from(0u8);
    assert_eq!(zero, Ok(Enum::Zero));

    const three: Result<Enum, ConstTryFromPrimitiveError<Enum>> = Enum::const_try_from(3u8);
    assert_eq!(
        three.unwrap_err().to_string(),
        "No discriminant in enum `Enum` matches the value `3`"
    );
}

#[test]
fn even() {
    #[derive(Debug, Eq, PartialEq, ConstTryFromPrimitive)]
    #[repr(u8)]
    enum Enum {
        Zero = 0,
        Two = 2,
        Four = 4,
    }

    const zero: Result<Enum, ConstTryFromPrimitiveError<Enum>> = Enum::const_try_from(0u8);
    assert_eq!(zero, Ok(Enum::Zero));

    const one: Result<Enum, ConstTryFromPrimitiveError<Enum>> = Enum::const_try_from(1u8);
    assert_eq!(
        one.unwrap_err().to_string(),
        "No discriminant in enum `Enum` matches the value `1`"
    );

    const two: Result<Enum, ConstTryFromPrimitiveError<Enum>> = Enum::const_try_from(2u8);
    assert_eq!(two, Ok(Enum::Two));

    const three: Result<Enum, ConstTryFromPrimitiveError<Enum>> = Enum::const_try_from(3u8);
    assert_eq!(
        three.unwrap_err().to_string(),
        "No discriminant in enum `Enum` matches the value `3`"
    );

    const four: Result<Enum, ConstTryFromPrimitiveError<Enum>> = Enum::const_try_from(4u8);
    assert_eq!(four, Ok(Enum::Four));
}

#[test]
fn skipped_value() {
    #[derive(Debug, Eq, PartialEq, ConstTryFromPrimitive)]
    #[repr(u8)]
    enum Enum {
        Zero,
        One,
        Three = 3,
        Four,
    }

    const zero: Result<Enum, ConstTryFromPrimitiveError<Enum>> = Enum::const_try_from(0u8);
    assert_eq!(zero, Ok(Enum::Zero));

    const one: Result<Enum, ConstTryFromPrimitiveError<Enum>> = Enum::const_try_from(1u8);
    assert_eq!(one, Ok(Enum::One));

    const two: Result<Enum, ConstTryFromPrimitiveError<Enum>> = Enum::const_try_from(2u8);
    assert_eq!(
        two.unwrap_err().to_string(),
        "No discriminant in enum `Enum` matches the value `2`"
    );

    const three: Result<Enum, ConstTryFromPrimitiveError<Enum>> = Enum::const_try_from(3u8);
    assert_eq!(three, Ok(Enum::Three));

    const four: Result<Enum, ConstTryFromPrimitiveError<Enum>> = Enum::const_try_from(4u8);
    assert_eq!(four, Ok(Enum::Four));
}

#[test]
fn wrong_order() {
    #[derive(Debug, Eq, PartialEq, ConstTryFromPrimitive)]
    #[repr(u8)]
    enum Enum {
        Four = 4,
        Three = 3,
        Zero = 0,
        One, // Zero + 1
    }

    const zero: Result<Enum, ConstTryFromPrimitiveError<Enum>> = Enum::const_try_from(0u8);
    assert_eq!(zero, Ok(Enum::Zero));

    const one: Result<Enum, ConstTryFromPrimitiveError<Enum>> = Enum::const_try_from(1u8);
    assert_eq!(one, Ok(Enum::One));

    const two: Result<Enum, ConstTryFromPrimitiveError<Enum>> = Enum::const_try_from(2u8);
    assert_eq!(
        two.unwrap_err().to_string(),
        "No discriminant in enum `Enum` matches the value `2`"
    );

    const three: Result<Enum, ConstTryFromPrimitiveError<Enum>> = Enum::const_try_from(3u8);
    assert_eq!(three, Ok(Enum::Three));

    const four: Result<Enum, ConstTryFromPrimitiveError<Enum>> = Enum::const_try_from(4u8);
    assert_eq!(four, Ok(Enum::Four));
}

#[test]
fn negative_values() {
    #[derive(Debug, Eq, PartialEq, ConstTryFromPrimitive)]
    #[repr(i8)]
    enum Enum {
        MinusTwo = -2,
        MinusOne = -1,
        Zero = 0,
        One = 1,
        Two = 2,
    }

    const minus_two: Result<Enum, ConstTryFromPrimitiveError<Enum>> = Enum::const_try_from(-2i8);
    assert_eq!(minus_two, Ok(Enum::MinusTwo));

    const minus_one: Result<Enum, ConstTryFromPrimitiveError<Enum>> = Enum::const_try_from(-1i8);
    assert_eq!(minus_one, Ok(Enum::MinusOne));

    const zero: Result<Enum, ConstTryFromPrimitiveError<Enum>> = Enum::const_try_from(0i8);
    assert_eq!(zero, Ok(Enum::Zero));

    const one: Result<Enum, ConstTryFromPrimitiveError<Enum>> = Enum::const_try_from(1i8);
    assert_eq!(one, Ok(Enum::One));

    const two: Result<Enum, ConstTryFromPrimitiveError<Enum>> = Enum::const_try_from(2i8);
    assert_eq!(two, Ok(Enum::Two));
}

#[test]
fn discriminant_expressions() {
    const ONE: u8 = 1;

    #[derive(Debug, Eq, PartialEq, ConstTryFromPrimitive)]
    #[repr(u8)]
    enum Enum {
        Zero,
        One = ONE,
        Two,
        Four = 4u8,
        Five,
        Six = ONE + ONE + 2u8 + 2,
    }

    const zero: Result<Enum, ConstTryFromPrimitiveError<Enum>> = Enum::const_try_from(0u8);
    assert_eq!(zero, Ok(Enum::Zero));

    const one: Result<Enum, ConstTryFromPrimitiveError<Enum>> = Enum::const_try_from(1u8);
    assert_eq!(one, Ok(Enum::One));

    const two: Result<Enum, ConstTryFromPrimitiveError<Enum>> = Enum::const_try_from(2u8);
    assert_eq!(two, Ok(Enum::Two));

    const three: Result<Enum, ConstTryFromPrimitiveError<Enum>> = Enum::const_try_from(3u8);
    assert_eq!(
        three.unwrap_err().to_string(),
        "No discriminant in enum `Enum` matches the value `3`",
    );

    const four: Result<Enum, ConstTryFromPrimitiveError<Enum>> = Enum::const_try_from(4u8);
    assert_eq!(four, Ok(Enum::Four));

    const five: Result<Enum, ConstTryFromPrimitiveError<Enum>> = Enum::const_try_from(5u8);
    assert_eq!(five, Ok(Enum::Five));

    const six: Result<Enum, ConstTryFromPrimitiveError<Enum>> = Enum::const_try_from(6u8);
    assert_eq!(six, Ok(Enum::Six));
}

#[cfg(feature = "complex-expressions")]
mod complex {
    use num_enum::ConstTryFromPrimitive;
    use std::convert::TryInto;

    const ONE: u8 = 1;

    #[derive(Debug, Eq, PartialEq, ConstTryFromPrimitive)]
    #[repr(u8)]
    enum Enum {
        Zero,
        One = ONE,
        Two,
        Four = 4u8,
        Five,
        Six = ONE + ONE + 2u8 + 2,
        Seven = (7, 2).0,
    }

    #[test]
    fn different_values() {
        const zero: Result<Enum, ConstTryFromPrimitiveError<Enum>> = Enum::const_try_from(0u8);
        assert_eq!(zero, Ok(Enum::Zero));

        const one: Result<Enum, ConstTryFromPrimitiveError<Enum>> = Enum::const_try_from(1u8);
        assert_eq!(one, Ok(Enum::One));

        const two: Result<Enum, ConstTryFromPrimitiveError<Enum>> = Enum::const_try_from(2u8);
        assert_eq!(two, Ok(Enum::Two));

        const three: Result<Enum, ConstTryFromPrimitiveError<Enum>> = Enum::const_try_from(3u8);
        assert_eq!(
            three.unwrap_err().to_string(),
            "No discriminant in enum `Enum` matches the value `3`",
        );

        const four: Result<Enum, ConstTryFromPrimitiveError<Enum>> = Enum::const_try_from(4u8);
        assert_eq!(four, Ok(Enum::Four));

        const five: Result<Enum, ConstTryFromPrimitiveError<Enum>> = Enum::const_try_from(5u8);
        assert_eq!(five, Ok(Enum::Five));

        const six: Result<Enum, ConstTryFromPrimitiveError<Enum>> = Enum::const_try_from(6u8);
        assert_eq!(six, Ok(Enum::Six));

        const seven: Result<Enum, ConstTryFromPrimitiveError<Enum>> = Enum::const_try_from(7u8);
        assert_eq!(seven, Ok(Enum::Seven));
    }

    #[derive(Debug, Eq, PartialEq, ConstTryFromPrimitive)]
    #[repr(u8)]
    enum EnumWithExclusiveRange {
        Zero = 0,
        #[num_enum(alternatives = [2..4])]
        OneOrTwoOrThree,
    }

    #[test]
    fn different_values_with_exclusive_range() {
        const zero: Result<EnumWithExclusiveRange, _> = Enum::const_try_from(0u8);
        assert_eq!(zero, Ok(EnumWithExclusiveRange::Zero));

        const one: Result<EnumWithExclusiveRange, _> = Enum::const_try_from(1u8);
        assert_eq!(one, Ok(EnumWithExclusiveRange::OneOrTwoOrThree));

        const two: Result<EnumWithExclusiveRange, _> = Enum::const_try_from(2u8);
        assert_eq!(two, Ok(EnumWithExclusiveRange::OneOrTwoOrThree));

        const three: Result<EnumWithExclusiveRange, _> = Enum::const_try_from(3u8);
        assert_eq!(three, Ok(EnumWithExclusiveRange::OneOrTwoOrThree));

        const four: Result<EnumWithExclusiveRange, _> = Enum::const_try_from(4u8);
        assert_eq!(
            four.unwrap_err().to_string(),
            "No discriminant in enum `EnumWithExclusiveRange` matches the value `4`",
        );
    }

    #[derive(Debug, Eq, PartialEq, ConstTryFromPrimitive)]
    #[repr(u8)]
    enum EnumWithInclusiveRange {
        Zero = 0,
        #[num_enum(alternatives = [2..=3])]
        OneOrTwoOrThree,
    }

    #[test]
    fn different_values_with_inclusive_range() {
        const zero: Result<EnumWithInclusiveRange, _> = Enum::const_try_from(0u8);
        assert_eq!(zero, Ok(EnumWithInclusiveRange::Zero));

        const one: Result<EnumWithInclusiveRange, _> = Enum::const_try_from(1u8);
        assert_eq!(one, Ok(EnumWithInclusiveRange::OneOrTwoOrThree));

        const two: Result<EnumWithInclusiveRange, _> = Enum::const_try_from(2u8);
        assert_eq!(two, Ok(EnumWithInclusiveRange::OneOrTwoOrThree));

        const three: Result<EnumWithInclusiveRange, _> = Enum::const_try_from(3u8);
        assert_eq!(three, Ok(EnumWithInclusiveRange::OneOrTwoOrThree));

        const four: Result<EnumWithInclusiveRange, _> = Enum::const_try_from(4u8);
        assert_eq!(
            four.unwrap_err().to_string(),
            "No discriminant in enum `EnumWithInclusiveRange` matches the value `4`",
        );
    }
}

#[test]
fn missing_trailing_comma() {
    #[rustfmt::skip]
#[derive(Debug, Eq, PartialEq, ConstTryFromPrimitive)]
#[repr(u8)]
enum Enum {
    Zero,
    One
}

    const zero: Result<Enum, ConstTryFromPrimitiveError<Enum>> = Enum::const_try_from(0u8);
    assert_eq!(zero, Ok(Enum::Zero));

    const one: Result<Enum, ConstTryFromPrimitiveError<Enum>> = Enum::const_try_from(1u8);
    assert_eq!(one, Ok(Enum::One));

    const two: Result<Enum, ConstTryFromPrimitiveError<Enum>> = Enum::const_try_from(2u8);
    assert_eq!(
        two.unwrap_err().to_string(),
        "No discriminant in enum `Enum` matches the value `2`"
    );
}

#[test]
fn ignores_extra_attributes() {
    #[derive(Debug, Eq, PartialEq, ConstTryFromPrimitive)]
    #[allow(unused)]
    #[repr(u8)]
    enum Enum {
        Zero,
        #[allow(unused)]
        One,
    }

    const zero: Result<Enum, ConstTryFromPrimitiveError<Enum>> = Enum::const_try_from(0u8);
    assert_eq!(zero, Ok(Enum::Zero));

    const one: Result<Enum, ConstTryFromPrimitiveError<Enum>> = Enum::const_try_from(1u8);
    assert_eq!(one, Ok(Enum::One));

    const two: Result<Enum, ConstTryFromPrimitiveError<Enum>> = Enum::const_try_from(2u8);
    assert_eq!(
        two.unwrap_err().to_string(),
        "No discriminant in enum `Enum` matches the value `2`"
    );
}

#[test]
fn visibility_is_fine() {
    #[derive(Debug, Eq, PartialEq, ConstTryFromPrimitive)]
    #[repr(u8)]
    pub(crate) enum Enum {
        Zero,
        One,
    }

    const zero: Result<Enum, ConstTryFromPrimitiveError<Enum>> = Enum::const_try_from(0u8);
    assert_eq!(zero, Ok(Enum::Zero));

    const one: Result<Enum, ConstTryFromPrimitiveError<Enum>> = Enum::const_try_from(1u8);
    assert_eq!(one, Ok(Enum::One));

    const two: Result<Enum, ConstTryFromPrimitiveError<Enum>> = Enum::const_try_from(2u8);
    assert_eq!(
        two.unwrap_err().to_string(),
        "No discriminant in enum `Enum` matches the value `2`"
    );
}

#[test]
fn error_variant_is_allowed() {
    #[derive(Debug, Eq, PartialEq, ConstTryFromPrimitive)]
    #[repr(u8)]
    pub enum Enum {
        Ok,
        Error,
    }

    const ok: Result<Enum, ConstTryFromPrimitiveError<Enum>> = Enum::const_try_from(0u8);
    assert_eq!(ok, Ok(Enum::Ok));

    const err: Result<Enum, ConstTryFromPrimitiveError<Enum>> = Enum::const_try_from(1u8);
    assert_eq!(err, Ok(Enum::Error));

    const unknown: Result<Enum, ConstTryFromPrimitiveError<Enum>> = Enum::const_try_from(2u8);
    assert_eq!(
        unknown.unwrap_err().to_string(),
        "No discriminant in enum `Enum` matches the value `2`"
    );
}

#[test]
fn alternative_values() {
    #[derive(Debug, Eq, PartialEq, ConstTryFromPrimitive)]
    #[repr(i8)]
    enum Enum {
        Zero = 0,
        #[num_enum(alternatives = [-1, 2, 3])]
        OneTwoThreeOrMinusOne = 1,
    }

    const minus_one: Result<Enum, ConstTryFromPrimitiveError<Enum>> = Enum::const_try_from(-1i8);
    assert_eq!(minus_one, Ok(Enum::OneTwoThreeOrMinusOne));

    const zero: Result<Enum, ConstTryFromPrimitiveError<Enum>> = Enum::const_try_from(0i8);
    assert_eq!(zero, Ok(Enum::Zero));

    const one: Result<Enum, ConstTryFromPrimitiveError<Enum>> = Enum::const_try_from(1i8);
    assert_eq!(one, Ok(Enum::OneTwoThreeOrMinusOne));

    const two: Result<Enum, ConstTryFromPrimitiveError<Enum>> = Enum::const_try_from(2i8);
    assert_eq!(two, Ok(Enum::OneTwoThreeOrMinusOne));

    const three: Result<Enum, ConstTryFromPrimitiveError<Enum>> = Enum::const_try_from(3i8);
    assert_eq!(three, Ok(Enum::OneTwoThreeOrMinusOne));

    const four: Result<Enum, ConstTryFromPrimitiveError<Enum>> = Enum::const_try_from(4i8);
    assert_eq!(
        four.unwrap_err().to_string(),
        "No discriminant in enum `Enum` matches the value `4`"
    );
}

#[test]
fn default_value() {
    #[derive(Debug, Eq, PartialEq, ConstTryFromPrimitive)]
    #[repr(u8)]
    enum Enum {
        Zero = 0,
        One = 1,
        #[num_enum(default)]
        Other = 2,
    }

    const zero: Result<Enum, ConstTryFromPrimitiveError<Enum>> = Enum::const_try_from(0u8);
    assert_eq!(zero, Ok(Enum::Zero));

    const one: Result<Enum, ConstTryFromPrimitiveError<Enum>> = Enum::const_try_from(1u8);
    assert_eq!(one, Ok(Enum::One));

    const two: Result<Enum, ConstTryFromPrimitiveError<Enum>> = Enum::const_try_from(2u8);
    assert_eq!(two, Ok(Enum::Other));

    const max_value: Result<Enum, ConstTryFromPrimitiveError<Enum>> =
        Enum::const_try_from(u8::max_value());
    assert_eq!(
        max_value.unwrap_err().to_string(),
        "No discriminant in enum `Enum` matches the value `255`"
    );
}

#[test]
fn alternative_values_and_default_value() {
    #[derive(Debug, Eq, PartialEq, ConstTryFromPrimitive)]
    #[repr(u8)]
    enum Enum {
        #[num_enum(default)]
        Zero = 0,
        One = 1,
        #[num_enum(alternatives = [3])]
        TwoOrThree = 2,
        Four = 4,
    }

    const zero: Result<Enum, ConstTryFromPrimitiveError<Enum>> = Enum::const_try_from(0u8);
    assert_eq!(zero, Ok(Enum::Zero));

    const one: Result<Enum, ConstTryFromPrimitiveError<Enum>> = Enum::const_try_from(1u8);
    assert_eq!(one, Ok(Enum::One));

    const two: Result<Enum, ConstTryFromPrimitiveError<Enum>> = Enum::const_try_from(2u8);
    assert_eq!(two, Ok(Enum::TwoOrThree));

    const three: Result<Enum, ConstTryFromPrimitiveError<Enum>> = Enum::const_try_from(3u8);
    assert_eq!(three, Ok(Enum::TwoOrThree));

    const four: Result<Enum, ConstTryFromPrimitiveError<Enum>> = Enum::const_try_from(4u8);
    assert_eq!(four, Ok(Enum::Four));

    const five: Result<Enum, ConstTryFromPrimitiveError<Enum>> = Enum::const_try_from(5u8);
    assert_eq!(
        five.unwrap_err().to_string(),
        "No discriminant in enum `Enum` matches the value `5`"
    );
}

#[test]
fn try_from_primitive_number() {
    #[derive(Debug, Eq, PartialEq, ConstTryFromPrimitive)]
    #[repr(u8)]
    enum Enum {
        #[num_enum(default)]
        Whatever = 0,
    }

    // #[derive(FromPrimitive)] generates implementations for the following traits:
    //
    // - `ConstTryFromPrimitive<T>`
    // - `TryFrom<T>`

    const try_from_primitive: Result<Enum, ConstTryFromPrimitiveError<Enum>> =
        Enum::const_try_from(0_u8);
    assert_eq!(try_from_primitive, Ok(Enum::Whatever));

    const try_from: Result<Enum, ConstTryFromPrimitiveError<Enum>> = Enum::const_try_from(0_u8);
    assert_eq!(try_from, Ok(Enum::Whatever));
}

#[test]
fn custom_error() {
    #[derive(Debug, Eq, PartialEq, ConstTryFromPrimitive)]
    #[num_enum(error_type(name = CustomError, constructor = CustomError::new))]
    #[repr(u8)]
    enum FirstNumber {
        Zero,
        One,
        Two,
    }

    #[derive(Debug, Eq, PartialEq, ConstTryFromPrimitive)]
    #[num_enum(error_type(constructor = CustomError::new, name = CustomError))]
    #[repr(u8)]
    enum SecondNumber {
        Zero,
        One,
        Two,
    }

    #[derive(Debug, PartialEq, Eq)]
    struct CustomError {
        bad_value: u8,
    }

    impl CustomError {
        const fn new(value: u8) -> CustomError {
            CustomError { bad_value: value }
        }
    }

    let zero: Result<FirstNumber, _> = FirstNumber::const_try_from(0u8);
    assert_eq!(zero, Ok(FirstNumber::Zero));

    let three: Result<FirstNumber, _> = FirstNumber::const_try_from(3u8);
    assert_eq!(three.unwrap_err(), CustomError { bad_value: 3u8 });

    let three: Result<SecondNumber, _> = SecondNumber::const_try_from(3u8);
    assert_eq!(three.unwrap_err(), CustomError { bad_value: 3u8 });
}

// #[derive(FromPrimitive)] generates implementations for the following traits:
//
// - `FromPrimitive<T>`
// - `From<T>`
// - `ConstTryFromPrimitive<T>`
// - `TryFrom<T>`
