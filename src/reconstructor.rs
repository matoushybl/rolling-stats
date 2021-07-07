use std::{io::ErrorKind, marker::PhantomData};

use crate::ConverterFromRaw;

pub struct Reconstructor<T, E> {
    _e: PhantomData<E>,
    intermediate_buffer: Vec<u8>,
    buffer: Vec<T>,
}

impl<T, E> Reconstructor<T, E> {
    pub fn new() -> Self {
        Self {
            _e: PhantomData,
            intermediate_buffer: Vec::new(),
            buffer: Vec::new(),
        }
    }

    pub fn data(&self) -> impl Iterator<Item = &T> {
        self.buffer.iter()
    }

    pub fn raw_data(&self) -> &[T] {
        &self.buffer
    }
}

impl<T, E> std::io::Write for Reconstructor<T, E>
where
    E: ConverterFromRaw<T>,
{
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

        if !chunks.remainder().is_empty() {
            // TODO push remainder to an intermediate buffer
            self.intermediate_buffer
                .extend_from_slice(chunks.remainder())
        }

        for value in chunks.map(|c| E::from_raw(c)) {
            let value = value.map_err(|_| {
                std::io::Error::new(ErrorKind::InvalidData, "Data conversion failed.")
            })?;
            self.buffer.push(value)
        }

        Ok(buf.len())
    }

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
        let mut reconstructor = Reconstructor::<i32, BigEndian>::new();
        let _ = reconstructor
            .write(&[0, 0, 0, 1, 0, 0, 0, 2, 0, 0, 0, 3, 0, 0, 0, 4])
            .unwrap();

        assert_eq!(reconstructor.raw_data(), &[1, 2, 3, 4]);
    }

    #[test]
    fn partial_data_leading() {
        let mut reconstructor = Reconstructor::<i32, BigEndian>::new();
        let _ = reconstructor.write(&[0, 0]).unwrap();
        let _ = reconstructor
            .write(&[0, 1, 0, 0, 0, 2, 0, 0, 0, 3, 0, 0, 0, 4])
            .unwrap();

        assert_eq!(reconstructor.raw_data(), &[1, 2, 3, 4]);
    }

    #[test]
    fn partial_data_trailing() {
        let mut reconstructor = Reconstructor::<i32, BigEndian>::new();
        let _ = reconstructor.write(&[0, 0, 0, 1, 0, 0]).unwrap();
        let _ = reconstructor
            .write(&[0, 2, 0, 0, 0, 3, 0, 0, 0, 4])
            .unwrap();

        assert_eq!(reconstructor.raw_data(), &[1, 2, 3, 4]);
    }
}
