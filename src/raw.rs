//! Abstraction of conversion of raw bytes into specific types.
//!
//! The abstraction is meant to be implemented by various types denoting raw data conversion - such as in this case types denoting big and little endian number representations.

use core::convert::TryInto;
use thiserror::Error;

/// The LittleEndian struct represents raw bytes conversion technique based on the Little Endian memory layout.
/// # Examples
/// ```
/// use rolling_stats::{LittleEndian, ConverterFromRaw};
///
/// let raw_data = [1u8, 0, 0, 0];
/// assert_eq!(1i32, LittleEndian::from_raw(&raw_data).unwrap());
/// ```
pub struct LittleEndian;

/// The BigEndian struct represents raw bytes conversion technique based on the Little Endian memory layout.
/// # Examples
/// ```
/// use rolling_stats::{BigEndian, ConverterFromRaw};
///
/// let raw_data = [0u8, 0, 0, 1];
/// assert_eq!(1i32, BigEndian::from_raw(&raw_data).unwrap());
/// ```
pub struct BigEndian;

/// Trait utilized for implementing conversion of raw bytes into specific types.
/// Implemented by Converter structs such as the `LittleEndian` and `BigEndian` structs.
/// `T` denotes the type the raw bytes should be converted into.
pub trait ConverterFromRaw<T> {
    /// Returns either the converted type from the raw input or an error.
    /// # Arguments
    /// * `raw` - raw bytes the type will be reconstructed from, length should be the same or longer than the type itself.
    fn from_raw(raw: &[u8]) -> Result<T, RawConversionError>;
}

/// An Error returned by the `ConverterFromRaw` trait on conversion failure.
#[derive(Clone, Copy, Debug, Error)]
pub enum RawConversionError {
    #[error("Not enough raw bytes were available for type conversion.")]
    NotEnoughData,
}

impl ConverterFromRaw<i32> for LittleEndian {
    fn from_raw(raw: &[u8]) -> Result<i32, RawConversionError> {
        if raw.len() < std::mem::size_of::<i32>() {
            return Err(RawConversionError::NotEnoughData);
        }

        Ok(i32::from_le_bytes(raw[..4].try_into().unwrap()))
    }
}

impl ConverterFromRaw<i32> for BigEndian {
    fn from_raw(raw: &[u8]) -> Result<i32, RawConversionError> {
        if raw.len() < std::mem::size_of::<i32>() {
            return Err(RawConversionError::NotEnoughData);
        }

        Ok(i32::from_be_bytes(raw[..4].try_into().unwrap()))
    }
}
