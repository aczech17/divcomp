use std::fs::{File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::ops::Range;
use crate::compress::DecompressionError;
use crate::compress::lz77::LONG_BUFFER_SIZE;

const BUFFER_SIZE: usize = LONG_BUFFER_SIZE;

pub struct DecompressionBuffer
{
    data: Vec<u8>,
    output_file: Option<File>,
}

impl DecompressionBuffer
{
    pub fn new() -> Result<DecompressionBuffer, DecompressionError>
    {
        let buffer = DecompressionBuffer
        {
            data: Vec::with_capacity(BUFFER_SIZE),
            output_file: None,
        };

        Ok(buffer)
    }

    pub fn set_output_file(&mut self, output_path: Option<&str>) -> Result<(), DecompressionError>
    {
        match output_path
        {
            None => self.output_file = None,
            Some(path) =>
            {
                let mut file = OpenOptions::new()
                    .read(true)
                    .write(true)
                    .create(true)
                    .open(path)
                    .map_err(|_| DecompressionError::Other)?;

                // Move the file pointer not to read the same data multiple times.
                // let file_offset = self.data.len() as u64;
                // println!("ustawiamy na {file_offset}");
                // file.seek(SeekFrom::Start(file_offset))
                //     .map_err(|_| DecompressionError::Other)?;

                self.output_file = Some(file);
            },
        };

        Ok(())
    }

    pub fn flush(&mut self, range: Range<usize>) -> Result<(), DecompressionError>
    {
        let bytes_to_remove: Vec<u8> = self.data.drain(range).collect(); // Remove bytes from the buffer.

        // If there is a file to write to, write the bytes to it.
        if let Some(file) = &mut self.output_file
        {
            file.write_all(&bytes_to_remove)
                .map_err(|_| DecompressionError::Other)?;
        }

        Ok(())
    }

    fn flush_all(&mut self) -> Result<(), DecompressionError>
    {
        let range = 0..self.data.len();
        self.flush(range)
    }

    fn read_reversily_from_file(&mut self, offset: usize, length: usize)
        -> Result<Vec<u8>, DecompressionError>
    {
        let file = match self.output_file.as_mut()
        {
            None =>
            {
                eprintln!("offset = {offset}, length = {length}, data size= {}", self.data.len());
                panic!();
            },
            Some(f) => f,
        };

        let mut bytes = vec![0; length];

        let offset_from_end = -(offset as i64);
        file.seek(SeekFrom::End(offset_from_end))
            .map_err(|_| DecompressionError::BadFormat)?;

        file.read_exact(&mut bytes)
            .map_err(|_| DecompressionError::Other)?;

        file.seek(SeekFrom::End(0))
            .map_err(|_| DecompressionError::Other)?;

        Ok(bytes)
    }

    pub(crate) fn push_byte(&mut self, value: u8) -> Result<(), DecompressionError>
    {
        // If we are writing to memory, don't flush.
        if !self.output_file.is_none() && self.data.len() == BUFFER_SIZE
        {
            self.flush_all()?;
        }

        self.data.push(value);
        Ok(())
    }

    fn get_bytes(&mut self, offset: usize, length: usize) -> Result<Vec<u8>, DecompressionError>
    {
        if offset <= self.data.len()
        {
            // All data are in buffer.
            let start = self.data.len() - offset;
            let end = start + length;
            let bytes = (&self.data[start..end]).to_vec();

            return Ok(bytes);
        }

        let first_index = self.data.len() as i64 - offset as i64;
        let last_index = first_index + length as i64 - 1;

        let file_offset = offset - self.data.len();

        if last_index < 0
        {
            // No needed data are in the buffer. Get all the data from the file.
            return self.read_reversily_from_file(file_offset, length);
        }

        // Get the data partially from the file and partially from the buffer.
        let length_in_file = file_offset; // Read the file to the end.
        let data_from_file = self.read_reversily_from_file(file_offset, length_in_file)?;

        let length_in_buffer = length - length_in_file;
        let data_from_buffer = (&self.data[0..length_in_buffer]).to_vec();

        let bytes = [data_from_file, data_from_buffer].concat();
        Ok(bytes)
    }

    /// Given couple (offset, length) it decompresses some bytes and returns
    /// count of decompressed bytes.
    pub fn decompress_couple(&mut self, offset: usize, length: usize)
        -> Result<usize, DecompressionError>
    {
        if offset == 0
        {
            return Ok(0);
        }

        let repeats_count = length / offset; // Sequence may be repeated...
        let reminder = length % offset;      // ...partially

        let mut decompressed_bytes = Vec::new();
        if repeats_count > 0
        {
            // A sequence is repeated. We copy the buffer from the offset to the end repeatedly
            // and we join it n times.

            let buffer_part = self.get_bytes(offset, offset)?;
            for _ in 0..repeats_count
            {
                decompressed_bytes.extend(buffer_part.iter().cloned());
            }
        }

        let mut buffer_part = self.get_bytes(offset, reminder)?;
        decompressed_bytes.append(&mut buffer_part);

        // Now we have a bunch of decompressed bytes. Let's push them to the buffer
        // and count them.
        let decompressed_bytes_count = decompressed_bytes.len();
        for byte in decompressed_bytes
        {
            self.push_byte(byte)?;
        }

        Ok(decompressed_bytes_count)
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

impl Drop for DecompressionBuffer
{
    fn drop(&mut self)
    {
        if let Err(_) = self.flush_all()
        {
            panic!("Could not flush the buffer while decompressing.");
        }
    }
}
