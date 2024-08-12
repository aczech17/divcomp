mod compression_window;
mod decompression_buffer;

use crate::compress::lz77::compression_window::CompressionWindow;
use crate::compress::{Compress, DecompressionError};

use std::fs::File;
use crate::compress::Decompress;
use crate::compress::lz77::decompression_buffer::DecompressionBuffer;
use crate::io_utils::byte_writer::ByteWriter;
use crate::io_utils::LZ77_SIGNATURE;
use crate::io_utils::universal_reader::UniversalReader;

const LONG_BUFFER_SIZE: usize = 1 << 16;
const SHORT_BUFFER_SIZE: usize = 258;

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
        let mut window = CompressionWindow::new(input);

        let output_file = File::create(output_filename)
            .map_err(|_| String::from("Could not create output file for LZ77."))?;

        let mut output = ByteWriter::new(output_file)?;

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

            // If no byte left, leave a trailing empty place.
            if let Some(byte) = byte_after
            {
                output.write_byte(byte);
            }

            window.shift(match_size + 1);
        }

        Ok(())
    }
}

pub struct LZ77Decompressor
{
    decompression_buffer: DecompressionBuffer,
    bytes_decompressed: usize,
}

impl LZ77Decompressor
{
    pub fn new(input_file: File) -> Result<Self, DecompressionError>
    {
        let mut input = UniversalReader::new(input_file);
        let mut decompression_buffer = DecompressionBuffer::new()?;

        while let Some(offset) = Self::load_u16(&mut input)
        {
            let offset = offset as usize;
            let length = Self::load_u16(&mut input).unwrap() as usize;

            decompression_buffer.decompress_couple(offset, length)?;

            match input.read_byte()
            {
                Some(byte_after) => decompression_buffer.push_byte(byte_after)?,
                None => break,
            };
        }

        let decompressor = LZ77Decompressor
        {
            decompression_buffer,
            bytes_decompressed: 0,
        };

        Ok(decompressor)
    }

    fn load_u16(input: &mut UniversalReader) -> Option<u16>
    {
        let b1 = input.read_byte()?;
        let b2 = input.read_byte()?;

        let value = ((b1 as u16) << 8) | (b2 as u16);
        Some(value)
    }
}

impl Decompress for LZ77Decompressor
{
    fn decompress_bytes_to_memory(&mut self, bytes_to_get: usize)
        -> Result<Vec<u8>, DecompressionError>
    {
        let range = self.bytes_decompressed..self.bytes_decompressed + bytes_to_get;

        let bytes =
            self.decompression_buffer.get_slice_of_data(range)?;
        self.bytes_decompressed += bytes_to_get;

        Ok(bytes)
    }

    fn decompress_bytes_to_file(&mut self, output_filename: &str, bytes_to_get: usize)
        -> Result<(), DecompressionError>
    {
        let range = self.bytes_decompressed..self.bytes_decompressed + bytes_to_get;
        self.decompression_buffer.write_bytes_to_file(range, output_filename)?;
        self.bytes_decompressed += bytes_to_get;

        Ok(())
    }

    fn ignore(&mut self, bytes_count: usize) -> Result<(), DecompressionError>
    {
        let _ignored_data = self.decompress_bytes_to_memory(bytes_count)?;

        Ok(())
    }
}
