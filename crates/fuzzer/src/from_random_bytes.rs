// Copyright 2023-2024 CrabNebula Ltd., Alexandre Dang
// SPDX-License-Identifier: PolyForm-Noncommercial-1.0.0

use std::array::TryFromSliceError;
use std::{str::Utf8Error, string::FromUtf8Error};

/// Construct a type from arbitrary random bytes.
pub trait FromRandomBytes<'a> {
    type Output;
    type Error;
    fn from_random_bytes(bytes: &'a [u8]) -> Result<Self::Output, Self::Error>;
}

impl FromRandomBytes<'_> for String {
    type Output = String;
    type Error = FromUtf8Error;

    // Random data often produce invalid utf-8 characters, we don't want to invalidate too much of
    // them so we use the lossy conversion
    fn from_random_bytes(bytes: &[u8]) -> Result<Self::Output, Self::Error> {
        Ok(String::from_utf8_lossy(bytes).to_string())
    }
}

impl<'a> FromRandomBytes<'a> for &str {
    type Output = &'a str;
    type Error = Utf8Error;

    fn from_random_bytes(bytes: &'a [u8]) -> Result<Self::Output, Self::Error> {
        std::str::from_utf8(bytes)
    }
}

macro_rules! impl_int {
    ($($int:ty),+ $(,)?) => {
        $(
            impl FromRandomBytes<'_> for $int {
                type Output = $int;
                type Error = TryFromSliceError;
                fn from_random_bytes(bytes: &[u8]) -> Result<Self::Output, Self::Error> {
                    // TODO: this is generic but can certainly be improved
                    // performance-wise

                    // Init an array with biggest basic integer size (128 bits)
                    let mut bytes_array = [0u8; 16];
                    for (dst, src) in bytes_array.iter_mut().zip(bytes) {
                        *dst = *src
                    }
                    let nb_bytes = std::mem::size_of::<$int>();
                    let first_bytes = bytes_array[0..nb_bytes].try_into().unwrap();
                    Ok(<$int>::from_be_bytes(first_bytes))
                }
            }
        )*
    };
}

impl_int!(u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize, f32, f64);
