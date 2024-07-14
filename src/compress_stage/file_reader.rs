use std::fs::File;
use std::io::Read;
use crate::compress_stage::bit_vector::Bit;

const BUFFER_SIZE: usize = 1024;

pub struct FileReader
{
    file_handle: File,
    buffer: [u8; BUFFER_SIZE],
    bytes_in_buffer: usize,
    bytes_read_from_buffer: usize,
    bits_read_total: usize,
}

impl FileReader
{
    pub fn new(file_handle: File) -> FileReader
    {
        let file_reader = FileReader
        {
            file_handle,
            buffer: [0; BUFFER_SIZE],
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

impl Iterator for FileReader
{
    type Item = u8;

    fn next(&mut self) -> Option<Self::Item>
    {
        self.read_byte()
    }
}
