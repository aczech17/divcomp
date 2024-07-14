use std::fs::File;
use std::io::Write;

const BUFFER_SIZE: usize = 1024;

pub struct ByteWriter
{
    file_handle: File,
    buffer: [u8; BUFFER_SIZE],
    bytes_in_buffer: usize,
}

impl ByteWriter
{
    pub fn new(output_filename: &str) -> Result<ByteWriter, String>
    {
        let file_handle = File::create(output_filename)
            .map_err(|_| format!("Could not create file buffer for {}.", output_filename))?;


        let byte_buffer = ByteWriter
        {
            file_handle,
            buffer: [0; BUFFER_SIZE],
            bytes_in_buffer: 0,
        };

        Ok(byte_buffer)
    }

    pub fn flush(&mut self)
    {
        if self.bytes_in_buffer > 0
        {
            self.file_handle.write_all(&self.buffer[0..self.bytes_in_buffer])
                .unwrap();
        }
        self.bytes_in_buffer = 0;
    }

    pub fn write_byte(&mut self, byte: u8)
    {
        if self.bytes_in_buffer == BUFFER_SIZE
        {
            self.flush();
        }

        self.buffer[self.bytes_in_buffer] = byte;
        self.bytes_in_buffer += 1;
    }
}

