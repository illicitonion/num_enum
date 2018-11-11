num_enum
========

Procedural macros to make inter-operation between primitives and enums easier.

Turning an enum into a primitive
--------------------------------

```rust
extern crate num_enum;
use num_enum::IntoPrimitive;

#[derive(IntoPrimitive)]
#[repr(u8)]
enum Number {
    Zero,
    One,
}

#[test]
fn convert() {
    let zero: u8 = Number::Zero.into();
    assert_eq!(zero, 0u8);
}
```

`num_enum`'s `IntoPrimitive` is more type-safe than using `as`, because `as` will silently truncate - `num_enum` only derives `From` for exactly the discriminant type of the enum.

Turning a primitive into an enum with try_from (nightly only)
-------------------------------------------------------------

```rust
#![feature(try_from)]

extern crate num_enum;

use num_enum::TryFromPrimitive;
use std::convert::TryInto;

#[derive(Debug, Eq, PartialEq, TryFromPrimitive)]
#[repr(u8)]
enum Number {
    Zero,
    One,
}

#[test]
fn convert() {
    let zero: Number = 0u8.try_into().unwrap();
    assert_eq!(zero, Ok(Number::Zero));

    let three: Result<Number, String> = 3u8.try_into();
    assert_eq!(three, Err("No value in enum Number for value 3".to_owned()));
}
```

Turning a primitive into an enum (stable rust)
----------------------------------------------

Unfortunately, `try_from` is not yet stable (see https://github.com/rust-lang/rust/issues/33417), so `num_enum` can also derive a custom trait which can be used in stable rust. It's a little ugly, but it does the job! It adds a function called `try_into_{enum_name}` which you can use. If you're converting outside of the module where the enum was defined, you may also need to `use your_module::TryInto{enum_name};`.

```rust
extern crate num_enum;

use num_enum::CustomTryInto;

#[derive(Debug, Eq, PartialEq, CustomTryInto)]
#[repr(u8)]
enum Number {
    Zero,
    One,
}

#[test]
fn convert() {
    let zero: Result<Number, String> = 0u8.try_into_Number();
    assert_eq!(zero, Ok(Number::Zero));

    let three: Result<Number, String> = 3u8.try_into_Number();
    assert_eq!(three, Err("No value in enum Number for value 3".to_owned()));
}
```

Optional features
-----------------

Some enum values may be composed of complex expressions, for example:

```rust
enum Number {
    Zero: (0, 1).0,
    One: (0, 1),1,
}
```

To cut down on compile time, these are not supported by default, but if you enable the `complex-expressions` feature of your dependency on `num_enum`, these should start working.
