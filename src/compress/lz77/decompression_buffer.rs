use std::fs;
use crate::compress::lz77::LONG_BUFFER_SIZE;
use crate::compress::DecompressionError;
use crate::io_utils::get_tmp_file_name;
use std::fs::OpenOptions;
use std::io::{Read, Seek, SeekFrom, Write};
use std::ops::Range;

const BUFFER_SIZE: usize = 2 * LONG_BUFFER_SIZE;

pub struct DecompressionBuffer
{
    memory: Vec<u8>,
    reserve_buffer_name: String,
    buffer_size_total: usize,
}

impl DecompressionBuffer
{
    pub fn new() -> Result<DecompressionBuffer, DecompressionError>
    {
        let reserve_buffer_name = get_tmp_file_name()
            .map_err(|_| DecompressionError::Other)?;

        let buffer = DecompressionBuffer
        {
            memory: Vec::with_capacity(BUFFER_SIZE),
            reserve_buffer_name,
            buffer_size_total: 0,
        };

        println!("New decompression buffer");
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

            let repeated_buffer_part = &self.memory[self.memory.len() - offset..];
            for _ in 0..repeats_count
            {
                decompressed_bytes.extend(repeated_buffer_part.iter().cloned());
            }
        }

        // Now decompress the reminder part.
        let reminder_range_start = self.memory.len() - offset;
        let reminder_range_stop = reminder_range_start + reminder;
        let reminder_range = reminder_range_start..reminder_range_stop;
        let mut reminder_buffer_part = (&self.memory[reminder_range]).to_vec();
        decompressed_bytes.append(&mut reminder_buffer_part);

        // Now we have a bunch of decompressed bytes. Let's push them to the buffer.
        for byte in decompressed_bytes
        {
            self.push_byte(byte);
        }

        Ok(())
    }

    pub fn push_byte(&mut self, value: u8)
    {
        if self.memory.len() == BUFFER_SIZE
        {
            let mut reserve_file = OpenOptions::new()
                .write(true)
                .append(true)
                .create(true)
                .open(&self.reserve_buffer_name)
                .expect("Could not open the reserve file.");

            reserve_file.write_all(&self.memory)
                .expect("Could not write to the reserve file.");

            self.memory.clear();
        }

        self.memory.push(value);
        self.buffer_size_total += 1;
    }

    fn read_data_from_reserve_file(&self, offset_from_end: usize, length: usize) -> Vec<u8>
    {
        let mut file = OpenOptions::new()
            .read(true)
            .write(false)
            .create(false)
            .open(&self.reserve_buffer_name)
            .expect("Could not read from the reserve file.");

        let mut data = vec![0; length];

        let file_offset = -(offset_from_end as i64);
        file.seek(SeekFrom::End(file_offset))
            .unwrap();

        file.read_exact(&mut data)
            .unwrap();

        file.seek(SeekFrom::End(0))
            .unwrap();

        data
    }

    pub fn get_slice_of_data(&self, range: Range<usize>) -> Vec<u8>
    {
        let start = range.start;
        let bytes_in_file = self.buffer_size_total - self.memory.len();

        if bytes_in_file == 0 // All data is in the memory.
        {
            return (&self.memory[range]).to_vec();
        }

        let offset_in_file = bytes_in_file - start;
        if range.len() <= offset_in_file // All data is in the file.
        {
            return self.read_data_from_reserve_file(offset_in_file, range.len());
        }

        // Data is partially in the file, partially in the memory.

        // Read the file till the end.
        let length_in_file = offset_in_file;
        let data_from_file = self.read_data_from_reserve_file(offset_in_file, length_in_file);

        let length_in_memory = range.len() - length_in_file;
        let data_from_memory = (&self.memory[0..length_in_memory]).to_vec();

        [data_from_file, data_from_memory].concat()
    }

    pub fn buffer_size_total(&self) -> usize
    {
        self.buffer_size_total
    }
}

impl Drop for DecompressionBuffer
{
    fn drop(&mut self)
    {
        fs::remove_file(&self.reserve_buffer_name)
            .expect("Could not remove temporary reserve file for the buffer.");
    }
}

