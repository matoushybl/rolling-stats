//! Reconstructor serves as a raw data stream parser that deals with interrupted/incomplete data.
//! That means raw data streams which do not contain whole multiplies of the Type length.

use crate::ConverterFromRaw;
use std::{io::ErrorKind, marker::PhantomData};

/// Reconstructor is a structure that holds all of the intermediate buffers
/// when receiving data using the `std::io::Write` trait.
/// It depends on the raw converter represented by the type parameter `E`.
/// The output type is denoted T.
/// The intermediate result (parsed `T`s) are contained in the Reconstructor itself
/// and are than retrieved using an iterator and cleared via the `std::io::Write` `flush` method.
pub struct Reconstructor<T, E> {
    _e: PhantomData<E>,
    /// A buffer that stores leftower raw data.
    intermediate_buffer: Vec<u8>,
    /// A buffer that stores the parsed data.
    buffer: Vec<T>,
}

/// Creates an empty Reconstructor instance with both of the intermediate buffers empty.
impl<T, E> Default for Reconstructor<T, E> {
    fn default() -> Self {
        Self {
            _e: PhantomData,
            intermediate_buffer: Vec::new(),
            buffer: Vec::new(),
        }
    }
}

#[allow(unused)]
impl<T, E> Reconstructor<T, E> {
    /// Returns an iterator over references to the parsed data.
    pub fn data(&self) -> impl Iterator<Item = &T> {
        self.buffer.iter()
    }

    /// Returns the parsed data as a raw slice.
    pub fn raw_data(&self) -> &[T] {
        &self.buffer
    }
}

/// The trait `std::io::Write` represents the data input into the RollingStats structure (the Reconstructor).
/// The raw data are parsed using the specified `ConverterFromRaw<T>` and stored into a buffer.
/// In the case of any leftover data, these are stored into the intermediate buffer, where they are retrieved once new raw data is written.
/// The contents of the parsed data buffer can be cleared using the `flush` method.
impl<T, E> std::io::Write for Reconstructor<T, E>
where
    E: ConverterFromRaw<T>,
{
    /// Parses the raw data into the concrete types and stores them into the data buffer.
    /// # Returns
    /// Returns the number of processed raw bytes (should always be equel to the length of the input raw data),
    /// or returns an error from parsing the raw data.
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let type_size = std::mem::size_of::<T>();
        if (buf.len() + self.intermediate_buffer.len()) < type_size {
            self.intermediate_buffer.extend_from_slice(buf);
            return Ok(buf.len());
        }

        let offset = if !self.intermediate_buffer.is_empty() {
            type_size - self.intermediate_buffer.len()
        } else {
            0
        };

        if offset > 0 {
            let mut data = Vec::new();
            data.extend_from_slice(&self.intermediate_buffer);
            data.extend_from_slice(&buf[..offset]);

            self.buffer.push(E::from_raw(&data).unwrap());
            self.intermediate_buffer.clear();
        }

        let chunks = buf[offset..].chunks_exact(type_size);

        self.intermediate_buffer
            .extend_from_slice(chunks.remainder());

        for value in chunks.map(|c| E::from_raw(c)) {
            let value = value.map_err(|_| {
                std::io::Error::new(ErrorKind::InvalidData, "Data conversion failed.")
            })?;
            self.buffer.push(value)
        }

        Ok(buf.len())
    }

    /// Clears the data buffer.
    /// # Returns
    /// Doesn't return any error as there are no fallible operations.
    fn flush(&mut self) -> std::io::Result<()> {
        self.buffer.clear();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::raw::BigEndian;
    use std::io::Write;

    use super::*;

    #[test]
    fn works() {
        let mut reconstructor = Reconstructor::<i32, BigEndian>::default();
        let _ = reconstructor
            .write(&[0, 0, 0, 1, 0, 0, 0, 2, 0, 0, 0, 3, 0, 0, 0, 4])
            .unwrap();

        assert_eq!(reconstructor.raw_data(), &[1, 2, 3, 4]);
    }

    #[test]
    fn partial_data_leading() {
        let mut reconstructor = Reconstructor::<i32, BigEndian>::default();
        let _ = reconstructor.write(&[0, 0]).unwrap();
        let _ = reconstructor
            .write(&[0, 1, 0, 0, 0, 2, 0, 0, 0, 3, 0, 0, 0, 4])
            .unwrap();

        assert_eq!(reconstructor.raw_data(), &[1, 2, 3, 4]);
    }

    #[test]
    fn partial_data_trailing() {
        let mut reconstructor = Reconstructor::<i32, BigEndian>::default();
        let _ = reconstructor.write(&[0, 0, 0, 1, 0, 0]).unwrap();
        let _ = reconstructor
            .write(&[0, 2, 0, 0, 0, 3, 0, 0, 0, 4])
            .unwrap();

        assert_eq!(reconstructor.raw_data(), &[1, 2, 3, 4]);
    }
}
