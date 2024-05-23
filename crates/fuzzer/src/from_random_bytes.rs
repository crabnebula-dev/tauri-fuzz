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
    fn from_random_bytes(bytes: &[u8]) -> Result<Self::Output, Self::Error> {
        String::from_utf8(bytes.to_vec())
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
                    bytes.try_into().map(|b| <$int>::from_be_bytes(b))
                }
            }
        )*
    };
}

impl_int!(u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize, f32, f64);
