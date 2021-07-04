use core::marker::PhantomData;
use std::{collections::VecDeque, io::Write, ops::Add};

pub struct LittleEndian;
pub struct BigEndian;

pub struct RollingStats<T, E, const WINDOW_SIZE: usize> {
    _e: PhantomData<E>,
    buffer: VecDeque<T>,
}

impl<T, E, const WINDOW_SIZE: usize> Write for RollingStats<T, E, WINDOW_SIZE> where E: FromRaw<T> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        todo!()
    }

    fn flush(&mut self) -> std::io::Result<()> {
        todo!()
    }
}

impl<T, E, const WINDOW_SIZE: usize> RollingStats<T, E, WINDOW_SIZE> where E: FromRaw<T> {

}

impl<T, E, const WINDOW_SIZE: usize> RollingStats<T, E, WINDOW_SIZE> where T: Copy + Default + Add<T, Output = T> + LossyF32Convertible {
    fn mean(&self) -> f32 {
        self.buffer.iter().fold(T::default(), |acc, item| {let acc = acc + *item; acc}).convert() / WINDOW_SIZE as f32
    }
}

pub enum RawConversionError {
    NotEnoughData,
}

pub trait FromRaw<T> {
    type Error;
    fn from(raw: &[u8]) -> Result<T, Self::Error>;

    fn raw_len() -> usize;

}

impl FromRaw<i32> for LittleEndian {
    type Error = RawConversionError;

    fn from(raw: &[u8]) -> Result<i32, RawConversionError> {
        Err(RawConversionError::NotEnoughData)
    }

    fn raw_len() -> usize {
        4
    }
}

pub trait LossyF32Convertible {
    fn convert(&self) -> f32;
}

impl LossyF32Convertible for i32 {
    fn convert(&self) -> f32 {
        *self as f32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let mut buffer = VecDeque::new();
        buffer.push_back(5);
        buffer.push_back(5);
        buffer.push_back(5);

        let roller = RollingStats::<i32, LittleEndian, 3> { _e: PhantomData, buffer };

        assert_eq!(roller.mean(), 5.0);
    }
}
