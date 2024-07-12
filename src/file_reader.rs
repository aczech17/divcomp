use std::fs::File;
use std::io::Read;

const BUFFER_SIZE: usize = 1024;

pub struct FileReader
{
    file_handle: File,
    buffer: [u8; BUFFER_SIZE],
    bytes_in_buffer: usize,
    bytes_read_from_buffer: usize,
}

impl FileReader
{
    pub fn new(mut file_handle: File) -> FileReader
    {
        let mut buffer = [0; BUFFER_SIZE];
        let bytes_read_from_file = file_handle.read(&mut buffer)
            .unwrap();

        let file_reader = FileReader
        {
            file_handle,
            buffer,
            bytes_in_buffer: bytes_read_from_file,
            bytes_read_from_buffer: 0,
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
}

impl Iterator for FileReader
{
    type Item = u8;

    fn next(&mut self) -> Option<Self::Item>
    {
        self.read_byte()
    }
}
