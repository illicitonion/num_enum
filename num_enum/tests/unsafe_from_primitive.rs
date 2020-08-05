use num_enum::UnsafeFromPrimitive;

#[test]
fn has_unsafe_from_primitive_number() {
    #[derive(Debug, Eq, PartialEq, UnsafeFromPrimitive)]
    #[repr(u8)]
    enum Enum {
        Zero,
        One,
    }

    unsafe {
        assert_eq!(Enum::from_unchecked(0_u8), Enum::Zero);
        assert_eq!(Enum::from_unchecked(1_u8), Enum::One);
    }
}
