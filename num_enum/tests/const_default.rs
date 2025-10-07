#![allow(non_upper_case_globals)]

// Guard against https://github.com/illicitonion/num_enum/issues/27
mod alloc {}
mod core {}
mod num_enum {}
mod std {}

#[test]
fn default() {
    #[derive(Debug, Eq, PartialEq, ::num_enum::ConstDefault)]
    #[repr(u8)]
    enum Enum {
        #[allow(unused)]
        Zero = 0,
        #[num_enum(default)]
        NonZero = 1,
    }

    const nz: Enum = Enum::const_default();
    assert_eq!(Enum::NonZero, nz);
}

#[test]
fn default_standard_default_attribute() {
    #[derive(Debug, Eq, PartialEq, ::num_enum::ConstDefault)]
    #[repr(u8)]
    enum Enum {
        #[allow(unused)]
        Zero = 0,
        #[default]
        NonZero = 1,
    }

    const nz: Enum = Enum::const_default();
    assert_eq!(Enum::NonZero, nz);
}
