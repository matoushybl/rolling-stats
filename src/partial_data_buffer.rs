//! PartialDataBuffer represents a way of dealing with incomplete raw data.
//! It consumes a slice of the newly received data, saves the data that are required for completing a previously incomplete data,
//! parses them and returns a new slice, that contains valid data for further processing.
//! Trailing incomplete data are handled as well.
//! The new returned slice has appropriate size so that an integer number of values can be parsed using it.
//!
//! As opposed to the `Reconstructor`, this solution avoids pointless copies.

use crate::ConverterFromRaw;
use std::marker::PhantomData;

/// Implements the partial data buffer - handling of incomplete data in a stream of raw data.
/// # Type parameteres
/// * `T` - denotes the type that is supposed to be parsed from the raw data
/// * `E` - denotes the raw data conversion algorithm
pub struct PartialDataBuffer<T, E> {
    _e: PhantomData<E>,
    _t: PhantomData<T>,
    buffer: Vec<u8>,
}

impl<T, E> Default for PartialDataBuffer<T, E> {
    /// Creates an empty buffer.
    fn default() -> Self {
        Self {
            _e: PhantomData,
            _t: PhantomData,
            buffer: Vec::new(),
        }
    }
}

impl<T, E> PartialDataBuffer<T, E>
where
    E: ConverterFromRaw<T>,
    T: Clone,
{
    /// Consumes the input slice of raw data, if enough data is present to reconstruct the partially received data, the data is and returned.
    /// The raw data slice is stripped off of the leading bytes belonging to the previously received incomplete data, any trailing partial data is stored to the internal buffer.
    ///
    /// # Returns
    /// Returns a slice constructed by removing partial data from the raw data stream.
    /// The returned slice is free of both the leading and trailing partial data.
    /// The returned slice contains a an integer of the target type lengths.
    pub fn consume<'a>(&mut self, raw: &'a [u8]) -> (Option<T>, &'a [u8]) {
        if self.buffer.len() + raw.len() < self.type_size() {
            self.buffer.extend(raw);
            return (None, &[]);
        }

        let offset = if !self.buffer.is_empty() {
            self.type_size() - self.buffer.len()
        } else {
            0
        };

        let reconstructed_value = if offset > 0 {
            self.buffer.extend(&raw[..offset]);
            let result = E::from_raw(&self.buffer).unwrap();
            self.clear();
            Some(result)
        } else {
            None
        };

        let remainder = (raw.len() - offset) % self.type_size();
        if remainder > 0 {
            self.buffer.extend(&raw[(raw.len() - remainder)..]);
        }

        (reconstructed_value, &raw[offset..(raw.len() - remainder)])
    }

    /// Clears the inner buffer, discarding the contained data.
    pub fn clear(&mut self) {
        self.buffer.clear();
    }

    /// Returns the size in bytes of the type meant to be reconstructed from the raw data,
    pub fn type_size(&self) -> usize {
        std::mem::size_of::<T>()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::LittleEndian;

    #[test]
    fn works() {
        let mut buffer = PartialDataBuffer::<i32, LittleEndian>::default();

        let data = [0x1, 0x0];
        let (item, rest) = buffer.consume(&data);
        assert!(item.is_none());
        assert_eq!(rest, &[]);
        assert_eq!(buffer.buffer.len(), 2);

        let data = [0x00, 0x00];
        let (item, rest) = buffer.consume(&data);
        assert!(item.is_some());
        assert_eq!(rest, &[]);
        assert_eq!(buffer.buffer.len(), 0);

        let data = [0x01, 0x00, 0x00, 0x00, 0x02, 0x00];
        let (item, rest) = buffer.consume(&data);
        assert!(item.is_none());
        assert_eq!(rest.len(), 4);
        assert_eq!(buffer.buffer.len(), 2);

        let data = [0x01, 0x00, 0x00, 0x00, 0x02, 0x00];
        let (item, rest) = buffer.consume(&data);
        assert!(item.is_some());
        assert_eq!(rest.len(), 4);
        assert_eq!(buffer.buffer.len(), 0);

        let data = [0x01, 0x00, 0x00, 0x00, 0x02, 0x00];
        let (item, rest) = buffer.consume(&data);
        assert!(item.is_none());
        assert_eq!(rest.len(), 4);
        assert_eq!(buffer.buffer.len(), 2);
    }
}
