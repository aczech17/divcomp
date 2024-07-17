use std::fs::File;
use std::io::Write;
use crate::io_utils::bit_vector::{Bit, BitVector};

const BUFFER_SIZE: usize = 8 * (1 << 26);

pub struct BitVectorWriter
{
    file_handle: File,
    buffer: BitVector,
}

impl BitVectorWriter
{
    pub fn new(filename: &str) -> Option<BitVectorWriter>
    {
        let file_handle = match File::create(filename)
        {
            Ok(file) => file,
            Err(_) => return None,
        };

        let file_writer = BitVectorWriter
        {
            file_handle,
            buffer: BitVector::new(),
        };

        Some(file_writer)
    }

    fn flush(&mut self)
    {
        let data = self.buffer.get_data();
        self.file_handle.write_all(data).unwrap();
        self.buffer.clear();
    }

    pub fn write_bit(&mut self, bit: Bit)
    {
        self.buffer.push_bit(bit);

        if self.buffer.size() == BUFFER_SIZE
        {
            self.flush();
        }
    }

    pub fn write_bit_vector(&mut self, bit_vector: &BitVector)
    {
        for index in 0..bit_vector.size()
        {
            let bit = bit_vector.get_bit(index);
            self.write_bit(bit);
        }
    }

    pub fn bits_in_buffer_count(&self) -> usize
    {
        self.buffer.size()
    }
}

impl Drop for BitVectorWriter
{
    fn drop(&mut self)
    {
        self.flush();
    }
}
