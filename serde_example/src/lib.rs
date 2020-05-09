#![no_std]

use num_enum::{IntoPrimitive, TryFromPrimitive, UnsafeFromPrimitive};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, IntoPrimitive, Serialize, TryFromPrimitive, UnsafeFromPrimitive)]
#[repr(u8)]
pub enum Number {
    Zero,
    One,
}
