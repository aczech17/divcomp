use std::fs::File;
use std::io::Write;
use crate::io_utils::get_memory_buffers_size;

pub struct ByteWriter
{
    file_handle: File,
    buffer: Vec<u8>,
    buffer_size: usize,
    bytes_in_buffer: usize,
}

impl ByteWriter
{
    pub fn new(output_filename: &str) -> Result<ByteWriter, String>
    {
        let file_handle = File::create(output_filename)
            .map_err(|_| format!("Could not create file buffer for {}.", output_filename))?;
        let buffer_size = get_memory_buffers_size();


        let byte_buffer = ByteWriter
        {
            file_handle,
            buffer: vec![0; buffer_size],
            buffer_size,
            bytes_in_buffer: 0,
        };

        Ok(byte_buffer)
    }

    fn flush(&mut self)
    {
        if self.bytes_in_buffer > 0
        {
            self.file_handle.write_all(&self.buffer[0..self.bytes_in_buffer])
                .unwrap();
            self.bytes_in_buffer = 0;
        }
    }

    pub fn write_byte(&mut self, byte: u8)
    {
        if self.bytes_in_buffer == self.buffer_size
        {
            self.flush();
        }

        self.buffer[self.bytes_in_buffer] = byte;
        self.bytes_in_buffer += 1;
    }
}

impl Drop for ByteWriter
{
    fn drop(&mut self)
    {
        self.flush();
    }
}
