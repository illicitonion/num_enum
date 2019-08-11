num_enum
========

Procedural macros to make inter-operation between primitives and enums easier.
This crate is no_std compatible.

[![crates.io](https://img.shields.io/crates/v/num_enum.svg)](https://crates.io/crates/num_enum)
[![Documentation](https://docs.rs/num_enum/badge.svg)](https://docs.rs/num_enum)
[![Build Status](https://travis-ci.org/illicitonion/num_enum.svg?branch=master)](https://travis-ci.org/illicitonion/num_enum)

Turning an enum into a primitive
--------------------------------

```rust
use num_enum::IntoPrimitive;

#[derive(IntoPrimitive)]
#[repr(u8)]
enum Number {
    Zero,
    One,
}

fn main() {
    let zero: u8 = Number::Zero.into();
    assert_eq!(zero, 0u8);
}
```

`num_enum`'s `IntoPrimitive` is more type-safe than using `as`, because `as` will silently truncate - `num_enum` only derives `From` for exactly the discriminant type of the enum.

Attempting to turn a primitive into an enum with try_from
----------------------------------------------

```rust
use num_enum::TryFromPrimitive;
use std::convert::TryFrom;

#[derive(Debug, Eq, PartialEq, TryFromPrimitive)]
#[repr(u8)]
enum Number {
    Zero,
    One,
}

fn main() {
    let zero = Number::try_from(0u8);
    assert_eq!(zero, Ok(Number::Zero));

    let three = Number::try_from(3u8);
    assert_eq!(
        three.unwrap_err().to_string(),
        "No discriminant in enum `Number` matches the value `3`",
    );
}
```

Unsafely turning a primitive into an enum with from_unchecked
-------------------------------------------------------------

If you're really certain a conversion will succeed, and want to avoid a small amount of overhead, you can use unsafe
code to do this conversion. Unless you have data showing that the match statement generated in the `try_from` above is a
bottleneck for you, you should avoid doing this, as the unsafe code has potential to cause serious memory issues in
your program.

```rust
use num_enum::UnsafeFromPrimitive;

#[derive(Debug, Eq, PartialEq, UnsafeFromPrimitive)]
#[repr(u8)]
enum Number {
    Zero,
    One,
}

fn main() {
    assert_eq!(
        Number::Zero,
        unsafe { Number::from_unchecked(0_u8) },
    );
    assert_eq!(
        Number::One,
        unsafe { Number::from_unchecked(1_u8) },
    );
}

unsafe fn undefined_behavior() {
    let _ = Number::from_unchecked(2); // 2 is not a valid discriminant!
}
```

Optional features
-----------------

Some enum values may be composed of complex expressions, for example:

```rust
enum Number {
    Zero = (0, 1).0,
    One = (0, 1).1,
}
```

To cut down on compile time, these are not supported by default, but if you enable the `complex-expressions` feature of your dependency on `num_enum`, these should start working.
