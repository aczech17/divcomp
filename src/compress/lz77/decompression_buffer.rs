use std::fs::{File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
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
    pub fn new(output_filename: Option<&str>) -> Result<DecompressionBuffer, DecompressionError>
    {
        let file = match output_filename
        {
            Some(filename) =>
            {
                let file_handle = OpenOptions::new()
                    .read(true)
                    .write(true)
                    .create(true)
                    .open(filename)
                    .map_err(|_| DecompressionError::Other)?;
                Some(file_handle)
            },
            None => None,
        };

        let buffer = DecompressionBuffer
        {
            data: Vec::with_capacity(BUFFER_SIZE),
            output_file: file,
        };

        Ok(buffer)
    }

    fn flush(&mut self) -> Result<(), DecompressionError>
    {
        // If there is no file to write to, never flush.
        if let Some(file) = &mut self.output_file
        {
            file.write_all(&self.data)
                .map_err(|_| DecompressionError::Other)?;
            self.data.clear();
        }

        Ok(())
    }

    fn read_reversily_from_file(&mut self, offset: usize, length: usize)
        -> Result<Vec<u8>, DecompressionError>
    {
        let file = self.output_file.as_mut().unwrap(); // We assume there is a file if we call it.

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

    fn push_byte(&mut self, value: u8) -> Result<(), DecompressionError>
    {
        if self.data.len() == BUFFER_SIZE
        {
            self.flush()?;
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

    /// Given triple (offset, length, byte after) it decompresses some bytes and returns
    /// count of decompressed bytes.
    pub fn decompress_triple(&mut self, offset: usize, length: usize, byte_after: Option<u8>)
        -> Result<usize, DecompressionError>
    {
        let mut decompressed_bytes_count = 0;

        if offset > 0
        {
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
            decompressed_bytes_count += decompressed_bytes.len();
            for byte in decompressed_bytes
            {
                self.push_byte(byte)?;
            }
        }

        if let Some(byte) = byte_after
        {
            self.push_byte(byte)?;
            decompressed_bytes_count += 1;
        }

        Ok(decompressed_bytes_count)
    }
}

impl Drop for DecompressionBuffer
{
    fn drop(&mut self)
    {
        if let Err(_) = self.flush()
        {
            panic!("Could not flush the buffer while decompressing.");
        }
    }
}
