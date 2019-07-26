mod into_primitive {
    use num_enum::IntoPrimitive;

    #[derive(IntoPrimitive)]
    #[repr(u8)]
    enum SimpleNumber {
        Zero,
        One,
        Two,
    }

    #[test]
    fn simple() {
        let zero: u8 = SimpleNumber::Zero.into();
        assert_eq!(zero, 0u8);

        let one: u8 = SimpleNumber::One.into();
        assert_eq!(one, 1u8);

        let two: u8 = SimpleNumber::Two.into();
        assert_eq!(two, 2u8);
    }
}

use num_enum::TryFromPrimitive;
use std::convert::TryInto;

#[derive(Debug, Eq, PartialEq, TryFromPrimitive)]
#[repr(u8)]
enum SimpleNumber {
    Zero,
    One,
    Two,
}

#[test]
fn simple() {
    let zero: Result<SimpleNumber, String> = 0u8.try_into();
    assert_eq!(zero, Ok(SimpleNumber::Zero));

    let three: Result<SimpleNumber, String> = 3u8.try_into();
    assert_eq!(
        three,
        Err("No value in enum SimpleNumber for value 3".to_owned())
    );
}

#[derive(Debug, Eq, PartialEq, TryFromPrimitive)]
#[repr(u8)]
enum EvenNumber {
    Zero = 0,
    Two = 2,
    Four = 4,
}

#[test]
fn even() {
    let zero: Result<EvenNumber, String> = 0u8.try_into();
    assert_eq!(zero, Ok(EvenNumber::Zero));

    let one: Result<EvenNumber, String> = 1u8.try_into();
    assert_eq!(
        one,
        Err("No value in enum EvenNumber for value 1".to_owned())
    );

    let two: Result<EvenNumber, String> = 2u8.try_into();
    assert_eq!(two, Ok(EvenNumber::Two));

    let three: Result<EvenNumber, String> = 3u8.try_into();
    assert_eq!(
        three,
        Err("No value in enum EvenNumber for value 3".to_owned())
    );

    let four: Result<EvenNumber, String> = 4u8.try_into();
    assert_eq!(four, Ok(EvenNumber::Four));
}

#[derive(Debug, Eq, PartialEq, TryFromPrimitive)]
#[repr(u8)]
enum SkippedNumber {
    Zero,
    One,
    Three = 3,
    Four,
}

#[test]
fn skipped_value() {
    let zero: Result<SkippedNumber, String> = 0u8.try_into();
    assert_eq!(zero, Ok(SkippedNumber::Zero));

    let one: Result<SkippedNumber, String> = 1u8.try_into();
    assert_eq!(one, Ok(SkippedNumber::One));

    let two: Result<SkippedNumber, String> = 2u8.try_into();
    assert_eq!(
        two,
        Err("No value in enum SkippedNumber for value 2".to_owned())
    );

    let three: Result<SkippedNumber, String> = 3u8.try_into();
    assert_eq!(three, Ok(SkippedNumber::Three));

    let four: Result<SkippedNumber, String> = 4u8.try_into();
    assert_eq!(four, Ok(SkippedNumber::Four));
}

#[derive(Debug, Eq, PartialEq, TryFromPrimitive)]
#[repr(u8)]
enum WrongOrderNumber {
    Four = 4,
    Three = 3,
    Zero = 0,
    One = 1,
}

#[test]
fn wrong_order() {
    let zero: Result<WrongOrderNumber, String> = 0u8.try_into();
    assert_eq!(zero, Ok(WrongOrderNumber::Zero));

    let one: Result<WrongOrderNumber, String> = 1u8.try_into();
    assert_eq!(one, Ok(WrongOrderNumber::One));

    let two: Result<WrongOrderNumber, String> = 2u8.try_into();
    assert_eq!(
        two,
        Err("No value in enum WrongOrderNumber for value 2".to_owned())
    );

    let three: Result<WrongOrderNumber, String> = 3u8.try_into();
    assert_eq!(three, Ok(WrongOrderNumber::Three));

    let four: Result<WrongOrderNumber, String> = 4u8.try_into();
    assert_eq!(four, Ok(WrongOrderNumber::Four));
}

#[cfg(feature = "complex-expression")]
mod complex {
    const ONE: u8 = 1;

    #[derive(Debug, Eq, PartialEq, TryFromPrimitive)]
    #[repr(u8)]
    enum DifferentValuesNumber {
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
        let zero: Result<DifferentValuesNumber, String> = 0u8.try_into();
        assert_eq!(zero, Ok(DifferentValuesNumber::Zero));

        let one: Result<DifferentValuesNumber, String> = 1u8.try_into();
        assert_eq!(one, Ok(DifferentValuesNumber::One));

        let two: Result<DifferentValuesNumber, String> = 2u8.try_into();
        assert_eq!(two, Ok(DifferentValuesNumber::Two));

        let three: Result<DifferentValuesNumber, String> = 3u8.try_into();
        assert_eq!(
            three,
            Err("No value in enum DifferentValuesNumber for value 3".to_owned())
        );

        let four: Result<DifferentValuesNumber, String> = 4u8.try_into();
        assert_eq!(four, Ok(DifferentValuesNumber::Four));

        let five: Result<DifferentValuesNumber, String> = 5u8.try_into();
        assert_eq!(five, Ok(DifferentValuesNumber::Five));

        let six: Result<DifferentValuesNumber, String> = 6u8.try_into();
        assert_eq!(six, Ok(DifferentValuesNumber::Six));

        let seven: Result<DifferentValuesNumber, String> = 7u8.try_into();
        assert_eq!(seven, Ok(DifferentValuesNumber::Seven));
    }
}

#[rustfmt::skip]
#[derive(Debug, Eq, PartialEq, TryFromPrimitive)]
#[repr(u8)]
enum MissingTrailingCommaNumber {
    Zero,
    One
}

#[test]
fn missing_trailing_comma() {
    let zero: Result<MissingTrailingCommaNumber, String> = 0u8.try_into();
    assert_eq!(zero, Ok(MissingTrailingCommaNumber::Zero));

    let one: Result<MissingTrailingCommaNumber, String> = 1u8.try_into();
    assert_eq!(one, Ok(MissingTrailingCommaNumber::One));

    let two: Result<MissingTrailingCommaNumber, String> = 2u8.try_into();
    assert_eq!(
        two,
        Err("No value in enum MissingTrailingCommaNumber for value 2".to_owned())
    );
}

#[derive(Debug, Eq, PartialEq, TryFromPrimitive)]
#[allow(unused)]
#[repr(u8)]
enum ExtraAttributes {
    Zero,
    #[allow(unused)]
    One,
}

#[test]
fn ignores_extra_attributes() {
    let zero: Result<ExtraAttributes, String> = 0u8.try_into();
    assert_eq!(zero, Ok(ExtraAttributes::Zero));

    let one: Result<ExtraAttributes, String> = 1u8.try_into();
    assert_eq!(one, Ok(ExtraAttributes::One));

    let two: Result<ExtraAttributes, String> = 2u8.try_into();
    assert_eq!(
        two,
        Err("No value in enum ExtraAttributes for value 2".to_owned())
    );
}

#[derive(Debug, Eq, PartialEq, TryFromPrimitive)]
#[repr(u8)]
pub(crate) enum VisibleNumber {
    Zero,
    One,
}

#[test]
fn visibility_is_fine() {
    let zero: Result<VisibleNumber, String> = 0u8.try_into();
    assert_eq!(zero, Ok(VisibleNumber::Zero));

    let one: Result<VisibleNumber, String> = 1u8.try_into();
    assert_eq!(one, Ok(VisibleNumber::One));

    let two: Result<VisibleNumber, String> = 2u8.try_into();
    assert_eq!(
        two,
        Err("No value in enum VisibleNumber for value 2".to_owned())
    );
}

#[derive(Debug, Eq, PartialEq, TryFromPrimitive)]
#[repr(u8)]
pub enum HasErrorVariant {
    Ok,
    Error,
}

#[test]
fn error_variant_is_allowed() {
    let ok: Result<HasErrorVariant, String> = 0u8.try_into();
    assert_eq!(ok, Ok(HasErrorVariant::Ok));

    let err: Result<HasErrorVariant, String> = 1u8.try_into();
    assert_eq!(err, Ok(HasErrorVariant::Error));

    let unknown: Result<HasErrorVariant, String> = 2u8.try_into();
    assert_eq!(
        unknown,
        Err("No value in enum HasErrorVariant for value 2".to_owned())
    );
}

use num_enum::UnsafeFromPrimitive;

#[derive(Debug, Eq, PartialEq, UnsafeFromPrimitive)]
#[repr(u8)]
enum HasUnsafeFromPrimitiveNumber {
    Zero,
    One,
}

#[test]
fn has_unsafe_from_primitive_number() {
    unsafe {
        assert_eq!(
            HasUnsafeFromPrimitiveNumber::Zero,
            HasUnsafeFromPrimitiveNumber::from(0_u8)
        );
        assert_eq!(
            HasUnsafeFromPrimitiveNumber::One,
            HasUnsafeFromPrimitiveNumber::from(1_u8)
        );
    }
}
