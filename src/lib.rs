//! Rolling stats is an implementation of a rolling buffer specified by a fixed size window, providing significant statistical values.
//! For dependency injection purposes, the statistics methods were abstracted away using the `Statistics` trait.
//!
//! # Basic use
//! ```
//! use rolling_stats::{LittleEndian, RollingStats, Statistics};
//! use std::io::Write;
//! use approx::*;
//!
//! let mut roller = RollingStats::<i32, LittleEndian, 3>::default();
//! let _ = roller
//!     .write(&[1, 0, 0, 0, 2, 0, 0, 0, 3, 0, 0, 0, 4, 0, 0, 0])
//!     .unwrap();
//! assert_abs_diff_eq!(roller.mean(), 3.0);
//! ```

mod convertf32;
mod partial_data_buffer;
mod raw;
mod reconstructor;

use core::marker::PhantomData;
use std::{collections::VecDeque, io::Write, ops::Add};

#[cfg(not(feature = "reconstructor"))]
use crate::partial_data_buffer::PartialDataBuffer;
use convertf32::LossyF32Convertible;
use rand_distr::{Distribution, Normal};
pub use raw::{BigEndian, ConverterFromRaw, LittleEndian};
#[cfg(feature = "reconstructor")]
use reconstructor::Reconstructor;

/// The `Statistics` trait useful for dependency injection.
/// This trait abstracts away basic statistics measures.
pub trait Statistics {
    /// Returns the mean of a dataset.
    fn mean(&self) -> f32;

    /// Returns standard deviation of a dataset.
    fn std_dev(&self) -> f32;

    /// Returns a number from a standard distribution specified by the mean and standard deviation of the dataset.
    fn rand(&self) -> f32;
}

/// Rolling stats is an implementation of a rolling buffer specified by a fixed size window, providing significant statistical values.
///
/// The raw data are written to the `RollingStats` using the `std::io::Write` trait.
/// As for handling partially received data, there are two choices, that can be enabled using a feature. The default one is more performant.
///
/// # Type parameters
/// * `T` - the type to be reconstructed from raw data.
/// * `E` - denotes a way to convert the raw data into the specified type
pub struct RollingStats<T, E, const WINDOW_SIZE: usize> {
    _e: PhantomData<E>,
    #[cfg(feature = "reconstructor")]
    reconstructor: Reconstructor<T, E>,
    #[cfg(not(feature = "reconstructor"))]
    intermediate_buffer: PartialDataBuffer<T, E>,
    buffer: VecDeque<T>,
}

impl<T, E, const WINDOW_SIZE: usize> RollingStats<T, E, WINDOW_SIZE> {
    /// Returns the number of items currently stored in the `RollingStats` struct.
    /// The maximal value returned is `WINDOW_SIZE`.
    pub fn len(&self) -> usize {
        self.buffer.len()
    }
}

#[cfg(feature = "reconstructor")]
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

#[cfg(not(feature = "reconstructor"))]
impl<T, E, const WINDOW_SIZE: usize> Write for RollingStats<T, E, WINDOW_SIZE>
where
    T: Copy,
    E: ConverterFromRaw<T>,
{
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let (reconstructed, remaining_buf) = self.intermediate_buffer.consume(&buf);
        if let Some(data) = reconstructed {
            self.buffer.push_back(data);
        }

        let parsed = remaining_buf
            .chunks_exact(std::mem::size_of::<T>())
            .map(|raw| E::from_raw(raw).unwrap());

        self.buffer.extend(parsed);
        while self.buffer.len() > WINDOW_SIZE {
            self.buffer.pop_front();
        }

        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

impl<T, E, const WINDOW_SIZE: usize> RollingStats<T, E, WINDOW_SIZE> {
    /// Creates a new instance of the `RollingStats` with empty buffer.
    pub fn new() -> Self {
        Self {
            _e: PhantomData,
            #[cfg(not(feature = "reconstructor"))]
            intermediate_buffer: Default::default(),
            #[cfg(feature = "reconstructor")]
            reconstructor: Reconstructor::default(),
            buffer: VecDeque::<T>::new(),
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
            / WINDOW_SIZE.min(self.buffer.len()).max(1) as f32
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
            #[cfg(not(feature = "reconstructor"))]
            intermediate_buffer: PartialDataBuffer::default(),
            #[cfg(feature = "reconstructor")]
            reconstructor: Reconstructor::default(),
            buffer,
        };

        assert_abs_diff_eq!(roller.mean(), 5.0);
    }

    #[test]
    fn test_partial_data() {
        let mut roller = RollingStats::<i32, BigEndian, 3>::default();
        let _ = roller.write(&[0, 0, 0]);

        assert_eq!(roller.len(), 0);
        assert_abs_diff_eq!(roller.mean(), 0.0);

        let _ = roller.write(&[1]);

        assert_eq!(roller.len(), 1);
        assert_abs_diff_eq!(roller.mean(), 1.0);

        let _ = roller.write(&[0, 0, 0, 2, 0]);

        assert_eq!(roller.len(), 2);
        assert_abs_diff_eq!(roller.mean(), 1.5);

        let _ = roller.write(&[0, 0, 3]);

        assert_eq!(roller.len(), 3);
        assert_abs_diff_eq!(roller.mean(), 2.0);
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
