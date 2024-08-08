use std::ops::Range;
use crate::compress::DecompressionError;
use crate::compress::lz77::LONG_BUFFER_SIZE;

const BUFFER_SIZE: usize = 2 * LONG_BUFFER_SIZE;

pub struct DecompressionBuffer
{
    data: Vec<u8>,
}

impl DecompressionBuffer
{
    pub fn new() -> Result<DecompressionBuffer, DecompressionError>
    {
        let buffer = DecompressionBuffer
        {
            data: Vec::with_capacity(BUFFER_SIZE),
        };

        Ok(buffer)
    }

    /// Given couple (offset, length), it decompresses some bytes and returns
    /// count of decompressed bytes.
    pub fn decompress_couple(&mut self, offset: usize, length: usize)
        -> Result<(), DecompressionError>
    {
        if offset == 0
        {
            return Ok(());
        }

        let repeats_count = length / offset; // Sequence may be repeated...
        let reminder = length % offset;      // ...partially

        let mut decompressed_bytes = Vec::new();
        if repeats_count > 0
        {
            // A sequence is repeated. We copy the buffer from the offset to the end repeatedly
            // and we join it n times.

            let repeated_buffer_part = &self.data[self.data.len() - offset..];
            for _ in 0..repeats_count
            {
                decompressed_bytes.extend(repeated_buffer_part.iter().cloned());
            }
        }

        // Now decompress the reminder part.
        let reminder_range_start = self.data.len() - offset;
        let reminder_range_stop = reminder_range_start + reminder;
        let reminder_range = reminder_range_start..reminder_range_stop;
        let mut reminder_buffer_part = (&self.data[reminder_range]).to_vec();
        decompressed_bytes.append(&mut reminder_buffer_part);

        // Now we have a bunch of decompressed bytes. Let's push them to the buffer.
        self.data.extend(decompressed_bytes);

        Ok(())
    }

    pub fn push_byte(&mut self, value: u8)
    {
        self.data.push(value);
    }

    pub fn get_slice_of_data(&self, range: Range<usize>) -> Vec<u8>
    {
        (&self.data[range]).to_vec()
    }

    pub fn get_data(&self) -> &Vec<u8>
    {
        &self.data
    }

    pub fn bytes_in_buffer(&self) -> usize
    {
        self.data.len()
    }
}

