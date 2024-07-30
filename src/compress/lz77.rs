use crate::compress::{Compress, DecompressError};

use std::fs::File;
use crate::compress::Decompress;
use crate::io_utils::byte_writer::ByteWriter;
use crate::io_utils::LZ77_SIGNATURE;
use crate::io_utils::universal_reader::UniversalReader;

const LONG_BUFFER_SIZE: usize = 7;
const SHORT_BUFFER_SIZE: usize = 6;

struct Window
{
    long_buffer: Vec<u8>,
    short_buffer: Vec<u8>,
    file_reader: UniversalReader,
}

impl Window
{
    fn new(mut file_reader: UniversalReader) -> Window
    {
        let mut short_buffer = Vec::new();
        for _ in 0..SHORT_BUFFER_SIZE
        {
            if let Some(byte) = file_reader.read_byte()
            {
                short_buffer.push(byte);
            }
            else
            {
                break;
            }
        }

        Window
        {
            long_buffer: Vec::new(),
            short_buffer,
            file_reader,
        }
    }

    fn shift(&mut self)
    {
        if self.long_buffer.len() == LONG_BUFFER_SIZE
        {
            self.long_buffer.pop();
        }

        if !self.short_buffer.is_empty()
        {
            self.long_buffer.insert(0, self.short_buffer.remove(0));
        }

        if let Some(new_byte) = self.file_reader.read_byte()
        {
            self.short_buffer.push(new_byte);
        }
    }

    fn shift_n_times(&mut self, n: usize)
    {
        for _ in 0..n
        {
            self.shift();
        }
    }

    fn short_buffer_is_empty(&self) -> bool
    {
        self.short_buffer.is_empty()
    }

    fn data(&self) -> Vec<u8>
    {
        let long_buffer: Vec<u8> = self.long_buffer.iter().rev().cloned().collect();
        let short_buffer = self.short_buffer.clone();

        [long_buffer, short_buffer].concat()
    }

    fn find_longest_prefix(&self)
        ->
        (
            usize,  // offset between pattern start and last match (0 if no match)
            usize,          // prefix size (0 if no match)
            Option<u8>,     // next byte after
        )
    {
        let data = self.data();
        let data = data.as_slice();

        let text_range = 0..self.long_buffer.len();
        let pattern_range =
            self.long_buffer.len()..self.long_buffer.len() + self.short_buffer.len();

        if text_range.is_empty()
        {
            return (0, 0, data.get(pattern_range.start).cloned());
        }

        if pattern_range.len() == 1
        {
            return (0, 0, data.get(pattern_range.start).cloned());
        }

        let pattern_len = pattern_range.end - pattern_range.start;

        for prefix_len in (1..=pattern_len - 1).rev() // Starting from the longest possible prefix.
        {
            let prefix = &data[pattern_range.start..pattern_range.start + prefix_len];

            for start_index in (text_range.start..text_range.end).rev() // Looking for the last occurence.
            {
                let potential_match = &data[start_index..start_index + prefix_len];
                if potential_match == prefix
                {
                    let next_after = data.get(pattern_range.start + prefix_len)
                        .cloned();
                    let index = pattern_range.start - start_index;

                    return (index, prefix_len, next_after);
                }
            }
        }

        let next = data.get(pattern_range.start).cloned();
        (0, 0, next)
    }
}

pub struct LZ77Compressor;

impl Compress for LZ77Compressor
{
    fn compress(&self, input_filename: &str, output_filename: &str) -> Result<(), String>
    {
        let input_file = File::open(input_filename)
            .map_err(|err| err.to_string())?;
        let input = UniversalReader::new(input_file);
        let mut window = Window::new(input);

        let mut output = ByteWriter::new(output_filename)?;

        let signature_bytes: Vec<u8> = LZ77_SIGNATURE.to_be_bytes()
            .into_iter()
            .skip_while(|&byte| byte == 0)
            .collect();

        for signature_byte in signature_bytes
        {
            output.write_byte(signature_byte);
        }

        while !window.short_buffer_is_empty()
        {
            let (offset, match_size, byte_after) =
                window.find_longest_prefix();

            output.write_byte(offset as u8);
            output.write_byte(match_size as u8);
            window.shift_n_times(match_size + 1);

            if let Some(byte) = byte_after
            {
                output.write_byte(byte);
            }
        }

        Ok(())
    }
}

pub struct LZ77Decompressor
{

}

#[allow(dead_code, unused_variables)]
impl LZ77Decompressor
{
    pub fn new(input_file: File) -> Result<Self, DecompressError>
    {
        let s = LZ77Decompressor{};

        Ok(s)
    }
}

#[allow(dead_code, unused_variables)]
impl Decompress for LZ77Decompressor
{
    fn decompress_bytes_to_memory(&mut self, bytes_to_get: usize) -> Result<Vec<u8>, DecompressError> {
        todo!()
    }

    fn decompress_bytes_to_file(&mut self, output_filename: &str, count: usize) -> Result<(), DecompressError> {
        todo!()
    }

    fn ignore(&mut self, bytes_count: usize) -> Result<(), DecompressError> {
        todo!()
    }
}

#[cfg(test)]
mod compression_test
{
    use crate::compress::Compress;
    use crate::compress::lz77::LZ77Compressor;

    #[test]
    fn test1()
    {
        let compressor = LZ77Compressor;
        compressor.compress("test1.txt", "output.bin").
            unwrap();

        assert_eq!(1, 1);
    }

    #[test]
    fn test2()
    {
        let compressor = LZ77Compressor;
        compressor.compress("test2.txt", "output.bin").
            unwrap();

        assert_eq!(1, 1);
    }

    #[test]
    fn test3()
    {
        let compressor = LZ77Compressor;
        compressor.compress("test3.txt", "output.bin").
            unwrap();

        assert_eq!(1, 1);
    }

    #[test]
    fn test4()
    {
        let compressor = LZ77Compressor;
        compressor.compress("test4.txt", "output.bin").
            unwrap();

        assert_eq!(1, 1);
    }

    #[test]
    fn test_hello()
    {
        let compressor = LZ77Compressor;
        compressor.compress("hello.txt", "output.bin").
            unwrap();

        assert_eq!(1, 1);
    }
}
