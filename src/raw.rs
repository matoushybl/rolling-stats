#[derive(Clone, Copy, Debug)]
pub enum RawConversionError {
    NotEnoughData,
}

use core::convert::TryInto;

use crate::{BigEndian, LittleEndian};

pub trait FromRaw<T> {
    fn from(raw: &[u8]) -> Result<T, RawConversionError>;
}

impl FromRaw<i32> for LittleEndian {
    fn from(raw: &[u8]) -> Result<i32, RawConversionError> {
        if raw.len() < std::mem::size_of::<i32>() {
            return Err(RawConversionError::NotEnoughData);
        }

        Ok(i32::from_le_bytes(raw[..4].try_into().unwrap()))
    }
}

impl FromRaw<i32> for BigEndian {
    fn from(raw: &[u8]) -> Result<i32, RawConversionError> {
        if raw.len() < std::mem::size_of::<i32>() {
            return Err(RawConversionError::NotEnoughData);
        }

        Ok(i32::from_be_bytes(raw[..4].try_into().unwrap()))
    }
}
