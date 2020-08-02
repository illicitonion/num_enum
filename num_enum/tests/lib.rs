use num_enum::{FromPrimitive, TryFromPrimitive, UnsafeFromPrimitive};
use std::convert::{TryInto, TryFrom};

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

#[derive(Debug, Eq, PartialEq, TryFromPrimitive)]
#[repr(u8)]
enum SimpleNumber {
    Zero,
    One,
    Two,
}

#[test]
fn simple() {
    let zero: Result<SimpleNumber, _> = 0u8.try_into();
    assert_eq!(zero, Ok(SimpleNumber::Zero));

    let three: Result<SimpleNumber, _> = 3u8.try_into();
    assert_eq!(
        three.unwrap_err().to_string(),
        "No discriminant in enum `SimpleNumber` matches the value `3`"
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
    let zero: Result<EvenNumber, _> = 0u8.try_into();
    assert_eq!(zero, Ok(EvenNumber::Zero));

    let one: Result<EvenNumber, _> = 1u8.try_into();
    assert_eq!(
        one.unwrap_err().to_string(),
        "No discriminant in enum `EvenNumber` matches the value `1`"
    );

    let two: Result<EvenNumber, _> = 2u8.try_into();
    assert_eq!(two, Ok(EvenNumber::Two));

    let three: Result<EvenNumber, _> = 3u8.try_into();
    assert_eq!(
        three.unwrap_err().to_string(),
        "No discriminant in enum `EvenNumber` matches the value `3`"
    );

    let four: Result<EvenNumber, _> = 4u8.try_into();
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
    let zero: Result<SkippedNumber, _> = 0u8.try_into();
    assert_eq!(zero, Ok(SkippedNumber::Zero));

    let one: Result<SkippedNumber, _> = 1u8.try_into();
    assert_eq!(one, Ok(SkippedNumber::One));

    let two: Result<SkippedNumber, _> = 2u8.try_into();
    assert_eq!(
        two.unwrap_err().to_string(),
        "No discriminant in enum `SkippedNumber` matches the value `2`"
    );

    let three: Result<SkippedNumber, _> = 3u8.try_into();
    assert_eq!(three, Ok(SkippedNumber::Three));

    let four: Result<SkippedNumber, _> = 4u8.try_into();
    assert_eq!(four, Ok(SkippedNumber::Four));
}

#[derive(Debug, Eq, PartialEq, TryFromPrimitive)]
#[repr(u8)]
enum WrongOrderNumber {
    Four = 4,
    Three = 3,
    Zero = 0,
    One, // Zero + 1
}

#[test]
fn wrong_order() {
    let zero: Result<WrongOrderNumber, _> = 0u8.try_into();
    assert_eq!(zero, Ok(WrongOrderNumber::Zero));

    let one: Result<WrongOrderNumber, _> = 1u8.try_into();
    assert_eq!(one, Ok(WrongOrderNumber::One));

    let two: Result<WrongOrderNumber, _> = 2u8.try_into();
    assert_eq!(
        two.unwrap_err().to_string(),
        "No discriminant in enum `WrongOrderNumber` matches the value `2`"
    );

    let three: Result<WrongOrderNumber, _> = 3u8.try_into();
    assert_eq!(three, Ok(WrongOrderNumber::Three));

    let four: Result<WrongOrderNumber, _> = 4u8.try_into();
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
        let zero: Result<DifferentValuesNumber, _> = 0u8.try_into();
        assert_eq!(zero, Ok(DifferentValuesNumber::Zero));

        let one: Result<DifferentValuesNumber, _> = 1u8.try_into();
        assert_eq!(one, Ok(DifferentValuesNumber::One));

        let two: Result<DifferentValuesNumber, _> = 2u8.try_into();
        assert_eq!(two, Ok(DifferentValuesNumber::Two));

        let three: Result<DifferentValuesNumber, _> = 3u8.try_into();
        assert_eq!(
            three.unwrap_err().to_string(),
            "No discriminant in enum `DifferentValuesNumber` matches the value `3`",
        );

        let four: Result<DifferentValuesNumber, _> = 4u8.try_into();
        assert_eq!(four, Ok(DifferentValuesNumber::Four));

        let five: Result<DifferentValuesNumber, _> = 5u8.try_into();
        assert_eq!(five, Ok(DifferentValuesNumber::Five));

        let six: Result<DifferentValuesNumber, _> = 6u8.try_into();
        assert_eq!(six, Ok(DifferentValuesNumber::Six));

        let seven: Result<DifferentValuesNumber, _> = 7u8.try_into();
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
    let zero: Result<MissingTrailingCommaNumber, _> = 0u8.try_into();
    assert_eq!(zero, Ok(MissingTrailingCommaNumber::Zero));

    let one: Result<MissingTrailingCommaNumber, _> = 1u8.try_into();
    assert_eq!(one, Ok(MissingTrailingCommaNumber::One));

    let two: Result<MissingTrailingCommaNumber, _> = 2u8.try_into();
    assert_eq!(
        two.unwrap_err().to_string(),
        "No discriminant in enum `MissingTrailingCommaNumber` matches the value `2`"
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
    let zero: Result<ExtraAttributes, _> = 0u8.try_into();
    assert_eq!(zero, Ok(ExtraAttributes::Zero));

    let one: Result<ExtraAttributes, _> = 1u8.try_into();
    assert_eq!(one, Ok(ExtraAttributes::One));

    let two: Result<ExtraAttributes, _> = 2u8.try_into();
    assert_eq!(
        two.unwrap_err().to_string(),
        "No discriminant in enum `ExtraAttributes` matches the value `2`"
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
    let zero: Result<VisibleNumber, _> = 0u8.try_into();
    assert_eq!(zero, Ok(VisibleNumber::Zero));

    let one: Result<VisibleNumber, _> = 1u8.try_into();
    assert_eq!(one, Ok(VisibleNumber::One));

    let two: Result<VisibleNumber, _> = 2u8.try_into();
    assert_eq!(
        two.unwrap_err().to_string(),
        "No discriminant in enum `VisibleNumber` matches the value `2`"
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
    let ok: Result<HasErrorVariant, _> = 0u8.try_into();
    assert_eq!(ok, Ok(HasErrorVariant::Ok));

    let err: Result<HasErrorVariant, _> = 1u8.try_into();
    assert_eq!(err, Ok(HasErrorVariant::Error));

    let unknown: Result<HasErrorVariant, _> = 2u8.try_into();
    assert_eq!(
        unknown.unwrap_err().to_string(),
        "No discriminant in enum `HasErrorVariant` matches the value `2`"
    );
}

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
            HasUnsafeFromPrimitiveNumber::from_unchecked(0_u8),
            HasUnsafeFromPrimitiveNumber::Zero
        );
        assert_eq!(
            HasUnsafeFromPrimitiveNumber::from_unchecked(1_u8),
            HasUnsafeFromPrimitiveNumber::One
        );
    }
}

#[derive(Debug, Eq, PartialEq, TryFromPrimitive)]
#[repr(u8)]
enum VariantAliases {
    Zero = 0,
    #[num_enum(aliases = [2, 3])]
    OneTwoOrThree = 1,
}

#[test]
fn alias_values() {
    type Enum = VariantAliases;

    let zero: Result<Enum, _> = 0u8.try_into();
    assert_eq!(zero, Ok(Enum::Zero));

    let one: Result<Enum, _> = 1u8.try_into();
    assert_eq!(one, Ok(Enum::OneTwoOrThree));

    let two: Result<Enum, _> = 2u8.try_into();
    assert_eq!(two, Ok(Enum::OneTwoOrThree));

    let three: Result<Enum, _> = 3u8.try_into();
    assert_eq!(three, Ok(Enum::OneTwoOrThree));

    let four: Result<Enum, _> = 4u8.try_into();
    assert_eq!(
        four.unwrap_err().to_string(),
        "No discriminant in enum `VariantAliases` matches the value `4`"
    );
}

#[derive(Debug, Eq, PartialEq, TryFromPrimitive)]
#[repr(u8)]
enum DefaultVariant {
    Zero = 0,
    One = 1,
    #[num_enum(default)]
    Other = 2,
}

#[test]
fn default_value() {
    type Enum = DefaultVariant;

    let zero: Result<Enum, _> = 0u8.try_into();
    assert_eq!(zero, Ok(Enum::Zero));

    let one: Result<Enum, _> = 1u8.try_into();
    assert_eq!(one, Ok(Enum::One));

    let two: Result<Enum, _> = 2u8.try_into();
    assert_eq!(two, Ok(Enum::Other));

    let max_value: Result<Enum, _> = u8::max_value().try_into();
    assert_eq!(max_value, Ok(Enum::Other));
}

#[derive(Debug, Eq, PartialEq, TryFromPrimitive)]
#[repr(u8)]
enum VariantAliasesAndDefaultVariant {
    #[num_enum(default)]
    Zero = 0,
    One = 1,
    #[num_enum(aliases = [3])]
    TwoOrThree = 2,
}

#[test]
fn alias_values_and_default_value() {
    type Enum = VariantAliasesAndDefaultVariant;

    let zero: Result<Enum, _> = 0u8.try_into();
    assert_eq!(zero, Ok(Enum::Zero));

    let one: Result<Enum, _> = 1u8.try_into();
    assert_eq!(one, Ok(Enum::One));

    let two: Result<Enum, _> = 2u8.try_into();
    assert_eq!(two, Ok(Enum::TwoOrThree));

    let three: Result<Enum, _> = 3u8.try_into();
    assert_eq!(three, Ok(Enum::TwoOrThree));

    let four: Result<Enum, _> = 4u8.try_into();
    assert_eq!(four, Ok(Enum::Zero));
}

#[derive(Debug, Eq, PartialEq, FromPrimitive)]
#[repr(u8)]
enum HasFromPrimitiveNumber {
    Zero = 0,
    #[num_enum(default)]
    NonZero = 1,
}

#[test]
fn has_from_primitive_number() {
    type Enum = HasFromPrimitiveNumber;

    let zero = Enum::from_primitive(0_u8);
    assert_eq!(zero, Enum::Zero);

    let one = Enum::from_primitive(1_u8);
    assert_eq!(one, Enum::NonZero);

    let two = Enum::from_primitive(2_u8);
    assert_eq!(two, Enum::NonZero);

    // #[derive(FromPrimitive)] also generates implementations for:
    //
    // - From<T>
    // - TryFromPrimitive<T>
    // - TryFrom<T>
    let from_primitive = Enum::from_primitive(0_u8);
    assert_eq!(from_primitive, Enum::Zero);

    let from = Enum::from(0_u8);
    assert_eq!(from, Enum::Zero);

    let try_from_primitive = Enum::try_from_primitive(0_u8);
    assert_eq!(try_from_primitive, Ok(Enum::Zero));

    let try_from = Enum::try_from(0_u8);
    assert_eq!(try_from, Ok(Enum::Zero));
}

mod failing_compiles {
    #[test]
    fn failing_compiles() {
        let t = trybuild::TestCases::new();
        t.compile_fail("tests/failing_examples/*.rs")
    }
}
