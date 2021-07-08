//!

mod convertf32;
mod raw;
mod reconstructor;

use core::marker::PhantomData;
use std::{collections::VecDeque, io::Write, ops::Add};

use convertf32::LossyF32Convertible;
use rand_distr::{Distribution, Normal};
use raw::ConverterFromRaw;
use reconstructor::Reconstructor;

pub trait Statistics {
    fn mean(&self) -> f32;
    fn std_dev(&self) -> f32;
    fn rand(&self) -> f32;
}

pub struct RollingStats<T, E, const WINDOW_SIZE: usize> {
    _e: PhantomData<E>,
    reconstructor: Reconstructor<T, E>,
    buffer: VecDeque<T>,
}

impl<T, E, const WINDOW_SIZE: usize> Write for RollingStats<T, E, WINDOW_SIZE>
where
    T: Copy,
    E: ConverterFromRaw<T>,
{
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let result = self.reconstructor.write(buf);

        self.buffer.extend(self.reconstructor.data());
        self.reconstructor.flush()?;
        while self.buffer.len() > WINDOW_SIZE {
            self.buffer.pop_front();
        }

        result
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.reconstructor.flush()
    }
}

impl<T, E, const WINDOW_SIZE: usize> RollingStats<T, E, WINDOW_SIZE> {
    pub fn new() -> Self {
        Self {
            _e: PhantomData,
            buffer: VecDeque::<T>::new(),
            reconstructor: Reconstructor::default(),
        }
    }
}

impl<T, E, const WINDOW_SIZE: usize> Default for RollingStats<T, E, WINDOW_SIZE> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T, E, const WINDOW_SIZE: usize> Statistics for RollingStats<T, E, WINDOW_SIZE>
where
    T: Copy + Default + Add<T, Output = T> + LossyF32Convertible,
{
    fn mean(&self) -> f32 {
        self.buffer
            .iter()
            .fold(T::default(), |acc, item| acc + *item)
            .convert()
            / WINDOW_SIZE.min(self.buffer.len()) as f32
    }

    fn std_dev(&self) -> f32 {
        let mean = self.mean();

        let sum = self
            .buffer
            .iter()
            .fold(0.0, |acc, item| acc + (item.convert() - mean).powi(2));

        let divisor = WINDOW_SIZE.min(self.buffer.len()).max(2) - 1;

        (sum / divisor as f32).sqrt()
    }

    fn rand(&self) -> f32 {
        let dist = Normal::new(self.mean(), self.std_dev()).unwrap();
        dist.sample(&mut rand::thread_rng())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::*;
    use raw::{BigEndian, LittleEndian};

    #[test]
    fn test_basic_functionality() {
        let mut buffer = VecDeque::new();
        buffer.push_back(5);
        buffer.push_back(5);
        buffer.push_back(5);

        let roller = RollingStats::<i32, LittleEndian, 3> {
            _e: PhantomData,
            buffer,
            reconstructor: Reconstructor::default(),
        };

        assert_abs_diff_eq!(roller.mean(), 5.0);
    }

    #[test]
    fn test_mean() {
        let mut roller = RollingStats::<i32, BigEndian, 3>::default();
        let _ = roller
            .write(&[0, 0, 0, 1, 0, 0, 0, 2, 0, 0, 0, 3, 0, 0, 0, 4])
            .unwrap();
        assert_abs_diff_eq!(roller.mean(), 3.0);
    }

    #[test]
    fn test_std_dev() {
        let mut roller = RollingStats::<i32, BigEndian, 3>::default();
        let _ = roller
            .write(&[0, 0, 0, 1, 0, 0, 0, 2, 0, 0, 0, 3, 0, 0, 0, 4])
            .unwrap();
        assert_abs_diff_eq!(roller.std_dev(), 1.0);
    }
}
