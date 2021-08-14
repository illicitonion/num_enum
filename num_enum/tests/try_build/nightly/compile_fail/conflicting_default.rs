#![feature(derive_default_enum)]

#[derive(Default, num_enum::Default)]
#[repr(u8)]
enum Number {
    #[default]
    Zero,
}

fn main() {
}
