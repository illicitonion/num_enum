use ::num_enum::IntoPrimitive;

// Guard against https://github.com/illicitonion/num_enum/issues/27
mod alloc {}
mod core {}
mod num_enum {}
mod std {}

#[derive(IntoPrimitive)]
#[repr(u8)]
enum Enum {
    Zero,
    One,
    Two,
}

#[test]
fn simple() {
    let zero: u8 = Enum::Zero.into();
    assert_eq!(zero, 0u8);

    let one: u8 = Enum::One.into();
    assert_eq!(one, 1u8);

    let two: u8 = Enum::Two.into();
    assert_eq!(two, 2u8);
}
