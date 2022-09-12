#[derive(Debug, Eq, PartialEq, ::num_enum::TryFromPrimitive)]
#[allow(unused)]
#[num_enum(crate = std)]
#[repr(u8)]
enum Enum {
    Zero,
    #[allow(unused)]
    One,
}
fn main() {}
