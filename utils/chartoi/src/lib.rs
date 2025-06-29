// see https://github.com/niklasf/rust-chartoi/blob/master/src/lib.rs but instead of bytes, chars
#![no_std]

extern crate num_traits;

use core::fmt;

use num_traits::{CheckedAdd, CheckedMul, FromPrimitive, Zero};

/// Error that can occur when trying to parse an integer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseIntegerError {
    kind: ParseIntegerErrorKind,
}

impl ParseIntegerError {
    /// The specific kind of error that occured.
    pub fn kind(&self) -> ParseIntegerErrorKind {
        self.kind
    }
}

/// Kinds of errors that can occur when trying to parse an integer.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ParseIntegerErrorKind {
    /// Cannot parse integer without digits.
    Empty,
    /// Invalid digit found.
    InvalidDigit,
    /// Integer too large to fit in target type.
    PosOverflow,
    /// Integer too small to fit in target type.
    NegOverflow,
}

impl fmt::Display for ParseIntegerError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(match self.kind {
            ParseIntegerErrorKind::Empty => "cannot parse integer without digits",
            ParseIntegerErrorKind::InvalidDigit => "invalid digit found",
            ParseIntegerErrorKind::PosOverflow => "integer too large to fit in target type",
            ParseIntegerErrorKind::NegOverflow => "integer too small to fit in target type",
        })
    }
}

/// Converts a byte slice in a given base to an integer. Signs are not allowed.
///
/// # Errors
///
/// Returns [`ParseIntegerError`] for any of the following conditions:
///
/// * `bytes` is empty
/// * not all characters of `bytes` are `0-9`, `a-z` or `A-Z`
/// * not all characters refer to digits in the given `radix`
/// * the number overflows `I`
///
/// # Panics
///
/// Panics if `radix` is not in the range `2..=36` (or in the pathological
/// case that there is no representation of `radix` in `I`).
///
/// # Examples
///
/// ```
/// # use chartoi::chartou_radix;
/// assert_eq!(Ok(255), chartou_radix(&['f','f'], 16));
/// assert_eq!(Ok(42), chartou_radix(&['1','0','1','0','1','0'], 2));
/// ```
///
/// [`ParseIntegerError`]: struct.ParseIntegerError.html
pub fn chartou_radix<I>(bytes: &[char], radix: u32) -> Result<I, ParseIntegerError>
where
    I: FromPrimitive + Zero + CheckedAdd + CheckedMul,
{
    assert!(
        (2..=36).contains(&radix),
        "radix must lie in the range 2..=36, found {}",
        radix
    );

    let base = I::from_u32(radix).expect("radix can be represented as integer");

    if bytes.is_empty() {
        return Err(ParseIntegerError {
            kind: ParseIntegerErrorKind::Empty,
        });
    }

    let mut result = I::zero();

    for &digit in bytes {
        let mul = result.checked_mul(&base);
        let x = match digit.to_digit(radix).and_then(I::from_u32) {
            Some(x) => x,
            None => {
                return Err(ParseIntegerError {
                    kind: ParseIntegerErrorKind::InvalidDigit,
                });
            }
        };
        result = match mul {
            Some(result) => result,
            None => {
                return Err(ParseIntegerError {
                    kind: ParseIntegerErrorKind::PosOverflow,
                });
            }
        };
        result = match result.checked_add(&x) {
            Some(result) => result,
            None => {
                return Err(ParseIntegerError {
                    kind: ParseIntegerErrorKind::PosOverflow,
                });
            }
        };
    }

    Ok(result)
}

/// Converts a byte slice to an integer. Signs are not allowed.
///
/// # Errors
///
/// Returns [`ParseIntegerError`] for any of the following conditions:
///
/// * `bytes` is empty
/// * not all characters of `bytes` are `0-9`
/// * the number overflows `I`
///
/// # Panics
///
/// Panics in the pathological case that there is no representation of `10`
/// in `I`.
///
/// # Examples
///
/// ```
/// # use chartoi::chartou;
/// assert_eq!(Ok(12345), chartou(&['1', '2', '3', '4', '5']));
/// assert!(chartou::<u8>(&['+', '1']).is_err()); // only chartoi allows signs
/// assert!(chartou::<u8>(&['2','5','6']).is_err()); // overflow
/// ```
///
/// [`ParseIntegerError`]: struct.ParseIntegerError.html
pub fn chartou<I>(bytes: &[char]) -> Result<I, ParseIntegerError>
where
    I: FromPrimitive + Zero + CheckedAdd + CheckedMul,
{
    chartou_radix(bytes, 10)
}
