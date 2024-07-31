mod window;

use crate::compress::lz77::window::Window;
use crate::compress::{Compress, DecompressError};

use std::fs::File;
use crate::compress::Decompress;
use crate::io_utils::byte_writer::ByteWriter;
use crate::io_utils::LZ77_SIGNATURE;
use crate::io_utils::universal_reader::UniversalReader;


pub struct LZ77Compressor;

impl LZ77Compressor
{
    fn write_usize_to_file(value: usize, byte_writer: &mut ByteWriter)
    {
        let value = value as u16;
        byte_writer.write_byte((value >> 8) as u8);
        byte_writer.write_byte((value & 0xFF) as u8);
    }
}

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

            Self::write_usize_to_file(offset, &mut output);
            Self::write_usize_to_file(match_size, &mut output);
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
