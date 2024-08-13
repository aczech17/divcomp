use std::fs;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom, Write};
use std::ops::Range;
use crate::io_utils::{create_tmp_file, FileInfo};
use crate::compress::DecompressionError;


#[cfg(debug_assertions)]
const BUFFER_SIZE: usize = 1;

#[cfg(not(debug_assertions))]
const BUFFER_SIZE: usize = 1 << 27;


pub struct DecompressionBuffer
{
    memory: Vec<u8>,
    reserve_file: Option<FileInfo>,
    buffer_size_total: usize,
}

impl DecompressionBuffer
{
    pub fn new() -> Result<DecompressionBuffer, DecompressionError>
    {
        let buffer = DecompressionBuffer
        {
            memory: Vec::with_capacity(BUFFER_SIZE),
            reserve_file: None,
            buffer_size_total: 0,
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

        let repeats_count = length / offset; // A sequence may be repeated...
        let reminder = length % offset;      // ...partially

        let mut decompressed_bytes = Vec::new();
        if repeats_count > 0
        {
            // A sequence is repeated. We copy the buffer from the offset to the end repeatedly
            // and we join it n times.

            let repeated_buffer_part =
                self.get_slice_of_data(self.buffer_size_total - offset..self.buffer_size_total)?;

            for _ in 0..repeats_count
            {
                decompressed_bytes.extend(repeated_buffer_part.iter().cloned());
            }
        }

        // Now decompress the reminder part.
        let reminder_range_start = self.buffer_size_total - offset;
        let reminder_range_stop = reminder_range_start + reminder;
        let reminder_range = reminder_range_start..reminder_range_stop;
        let mut reminder_buffer_part =
            self.get_slice_of_data(reminder_range)?;
        decompressed_bytes.append(&mut reminder_buffer_part);

        // Now we have a bunch of decompressed bytes. Let's push them to the buffer.
        for byte in decompressed_bytes
        {
            self.push_byte(byte)?;
        }

        Ok(())
    }

    fn dump_to_reserve_file(&mut self) -> Result<(), DecompressionError>
    {
        if self.reserve_file.is_none()
        {
            let file = create_tmp_file(".resbuf")
                .ok_or(DecompressionError::Other)?;

            self.reserve_file = Some(file);
        };

        let FileInfo {handle: file, .. } = self.reserve_file
            .as_mut()
            .unwrap();  // We are sure the file is already created, so we can unwrap.

        file.write_all(&self.memory)
            .expect("Could not write to the reserve file.");

        self.memory.clear();

        Ok(())
    }

    pub fn push_byte(&mut self, value: u8) -> Result<(), DecompressionError>
    {
        if self.memory.len() == BUFFER_SIZE
        {
            self.dump_to_reserve_file()?;
        }

        self.memory.push(value);
        self.buffer_size_total += 1;

        Ok(())
    }

    fn read_data_from_reserve_file(&mut self, offset_from_end: i64, length: usize)
        -> Result<Vec<u8>, DecompressionError>
    {
        let FileInfo{handle: file, .. } = self.reserve_file
            .as_mut()
            .unwrap();

        let mut data = vec![0; length];

        file.seek(SeekFrom::End(-offset_from_end))
            .unwrap();
        file.read_exact(&mut data).unwrap();
        file.seek(SeekFrom::End(0))
            .unwrap();

        Ok(data)
    }

    pub fn get_slice_of_data(&mut self, range: Range<usize>)
        -> Result<Vec<u8>, DecompressionError>
    {
        let start = range.start;
        let length = range.len();

        let bytes_in_memory = self.memory.len();
        let bytes_in_file = self.buffer_size_total - bytes_in_memory;

        let offset_in_file = (bytes_in_file as i64) - (start as i64);

        if offset_in_file <= 0 // All the needed data is in the memory.
        {
            let memory_start = start - bytes_in_file;
            let memory_end = memory_start + length;

            let data = self.memory[memory_start..memory_end].to_vec();
            return Ok(data);
        }

        if length <= offset_in_file as usize // All the needed data is in the file.
        {
            return self.read_data_from_reserve_file(offset_in_file, length);
        }

        // The data is partially in the file and partially in the memory.

        // Read the file to the end.
        let length_in_file = offset_in_file as usize;
        let data_from_file =
            self.read_data_from_reserve_file(offset_in_file, length_in_file)?;

        // Read the rest of the data from the memory.
        let length_in_memory = length - length_in_file;
        let data_from_memory = self.memory[0..length_in_memory].to_vec();

        // Join the 2 parts and return.
        let slice_of_data = [data_from_file, data_from_memory].concat();
        Ok(slice_of_data)
    }

    pub fn write_bytes_to_file(&mut self, range: Range<usize>, output_filename: &str)
        -> Result<(), DecompressionError>
    {
        let start = range.start;
        let length = range.len();

        let mut file = File::create(output_filename)
            .map_err(|_| DecompressionError::FileCreationError)?;

        let iterations = length / BUFFER_SIZE;
        for i in 0..iterations
        {
            let from = start + i * BUFFER_SIZE;
            let to = from + BUFFER_SIZE;
            let portion = self.get_slice_of_data(from..to)?;

            file.write_all(&portion)
                .map_err(|_| DecompressionError::Other)?;
        }

        let from = start + iterations * BUFFER_SIZE;
        let to = range.end;
        let portion = self.get_slice_of_data(from..to)?;

        file.write_all(&portion)
            .map_err(|_| DecompressionError::Other)
    }
}

impl Drop for DecompressionBuffer
{
    fn drop(&mut self)
    {
        if let Some(file_info) = &self.reserve_file
        {
            let FileInfo{path,..} = file_info;

            fs::remove_file(path)
                .expect("Could not remove temporary reserve file for the buffer.");
        }
    }
}

