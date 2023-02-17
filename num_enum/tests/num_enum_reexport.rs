extern crate std as num_enum; // ensure `::num_enum` is a bad path

mod reexport {
    pub extern crate num_enum;
}

#[test]
fn crate_path() {
    use std::convert::TryInto;
    #[derive(Debug, Eq, PartialEq, reexport::num_enum::TryFromPrimitive)]
    #[allow(unused)]
    #[num_enum(crate = reexport::num_enum)]
    #[repr(u8)]
    enum Enum {
        Zero,
        #[allow(unused)]
        One,
    }

    let zero: Result<Enum, _> = 0u8.try_into();
    assert_eq!(zero, Ok(Enum::Zero));
}
