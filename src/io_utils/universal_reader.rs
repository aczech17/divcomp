use std::fs::File;
use std::io::Read;
use crate::io_utils::bit_vector::Bit;

const BUFFER_SIZE: usize = 1 << 26;

pub struct UniversalReader
{
    file_handle: File,
    buffer: Vec<u8>,
    bytes_in_buffer: usize,
    bytes_read_from_buffer: usize,
    bits_read_total: usize,
}

impl UniversalReader
{
    pub fn new(file_handle: File) -> UniversalReader
    {
        let file_reader = UniversalReader
        {
            file_handle,
            buffer: vec![0; BUFFER_SIZE],
            bytes_in_buffer: 0,
            bytes_read_from_buffer: 0,
            bits_read_total: 0,
        };

        file_reader
    }

    fn refill_buffer(&mut self)
    {
        self.bytes_in_buffer = self.file_handle.read(&mut self.buffer)
            .unwrap();

        self.bytes_read_from_buffer = 0;
    }

    pub fn read_byte(&mut self) -> Option<u8>
    {
        if self.bytes_read_from_buffer == self.bytes_in_buffer
        {
            self.refill_buffer();
            if self.bytes_in_buffer == 0
            {
                return None;
            }
        }

        let data = self.buffer[self.bytes_read_from_buffer];
        self.bytes_read_from_buffer += 1;

        Some(data)
    }

    pub fn read_some_bytes(&mut self, count: usize) -> Result<Vec<u8>, ()>
    {
        (0..count)
            .map(|_| self.read_byte().ok_or(()))
            .collect()
    }

    pub fn read_bit(&mut self) -> Option<Bit>
    {
        if (self.bits_read_total) % (BUFFER_SIZE * 8) == 0
        {
            self.refill_buffer();
            if self.bytes_in_buffer == 0
            {
                return None;
            }
        }

        let bit_index_in_buffer = self.bits_read_total % (8 * BUFFER_SIZE);

        let byte_index_in_buffer = bit_index_in_buffer / 8;
        let bit_index_in_byte = bit_index_in_buffer % 8;

        let bit = (self.buffer[byte_index_in_buffer] >> (7 - bit_index_in_byte)) & 1;
        self.bits_read_total += 1;
        Some(bit)
    }

    pub fn bits_read(&self) -> usize
    {
        self.bits_read_total
    }
}

impl Iterator for UniversalReader
{
    type Item = u8;

    fn next(&mut self) -> Option<Self::Item>
    {
        self.read_byte()
    }
}
