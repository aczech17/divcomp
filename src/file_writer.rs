use std::fs::File;
use std::io::Write;
use crate::bit_vector::{Bit, BitVector};

const BUFFER_SIZE: usize = 1024;

pub struct FileWriter
{
    file_handle: File,
    buffer: BitVector,
}

impl FileWriter
{
    pub fn new(filename: &str) -> Option<FileWriter>
    {
        let file_handle = match File::create(filename)
        {
            Ok(file) => file,
            Err(_) => return None,
        };

        let file_writer = FileWriter
        {
            file_handle,
            buffer: BitVector::new(),
        };

        Some(file_writer)
    }

    pub fn dump_buffer(&mut self)
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
            self.dump_buffer();
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