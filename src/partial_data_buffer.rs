use std::marker::PhantomData;
use crate::ConverterFromRaw;

pub struct IntermediateBuffer<T, E> {
    _e: PhantomData<E>,
    _t: PhantomData<T>,
    buffer: Vec<u8>,
}

impl<T, E> Default for IntermediateBuffer<T, E> {
    fn default() -> Self {
        Self {
            _e: PhantomData,
            _t: PhantomData,
            buffer: Vec::new(),
        }
    }
}

impl<T, E> IntermediateBuffer<T, E>
    where
        E: ConverterFromRaw<T>,
        T: Clone,
{
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

    pub fn clear(&mut self) {
        self.buffer.clear();
    }

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
        let mut buffer = IntermediateBuffer::<i32, LittleEndian>::default();

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
