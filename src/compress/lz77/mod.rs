mod compression_window;
mod decompression_buffer;

use crate::compress::lz77::compression_window::CompressionWindow;
use crate::compress::{Compress, DecompressError};

use std::fs::File;
use crate::compress::Decompress;
use crate::io_utils::byte_writer::ByteWriter;
use crate::io_utils::LZ77_SIGNATURE;
use crate::io_utils::universal_reader::UniversalReader;

const LONG_BUFFER_SIZE: usize = 1 << 15;
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

}

impl LZ77Decompressor
{
    pub fn new(input_file: File) -> Result<Self, DecompressError>
    {
        let s = LZ77Decompressor{};

        Ok(s)
    }
}

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
    use std::fs;
    use std::fs::File;
    use std::io::Read;
    use crate::compress::{Compress, Decompress};
    use crate::compress::lz77::{LZ77Compressor, LZ77Decompressor};
    use crate::io_utils::get_tmp_file_name;

    fn do_test(filename: &str)
    {
        let compressed_file_name = get_tmp_file_name()
            .unwrap();

        let compressor = LZ77Compressor;
        compressor.compress(filename, &compressed_file_name)
            .unwrap();
        let compressed_file_size = fs::metadata(&compressed_file_name).unwrap()
            .len() as usize;


        let decompressed_file_name = get_tmp_file_name()
            .unwrap();
        let mut decompressor = LZ77Decompressor{};
        decompressor.decompress_bytes_to_file(&decompressed_file_name, compressed_file_size)
            .unwrap();

        let mut input_file = File::open(filename)
            .unwrap();
        let file_size = input_file.metadata().unwrap().len() as usize;
        let mut decompressed_file = File::open(&decompressed_file_name)
            .unwrap();

        let mut input_file_content: Vec<u8> = Vec::with_capacity(file_size);
        let mut decompressed_file_content: Vec<u8> = Vec::with_capacity(file_size);

        input_file.read(&mut input_file_content)
            .unwrap();
        decompressed_file.read(&mut decompressed_file_content)
            .unwrap();

        let files_are_equal = input_file_content == decompressed_file_content;

        fs::remove_file(&compressed_file_name)
            .unwrap();
        fs::remove_file(&decompressed_file_name)
            .unwrap();

        assert!(files_are_equal);
    }

    #[test]
    fn multiple_as()
    {
        do_test("test/aaa.txt");
    }

    #[test]
    fn KOKOKOKO()
    {
        do_test("test/KOKOKOKO.txt");
    }

    #[test]
    fn KOKOKOKOEUROSPOKO()
    {
        do_test("test/KOKOKOKOEUROSPOKO.txt")
    }

    #[test]
    fn KOKOKOKOEE()
    {
        do_test("test/KOKOKOKOEE.txt");
    }

    #[test]
    fn KOKOKOKOEEE()
    {
        do_test("test/KOKOKOKOEEE.txt");
    }

    // #[test]
    // fn test1()
    // {
    //     let compressor = LZ77Compressor;
    //     compressor.compress("test1.txt", "output.bin").
    //         unwrap();
    //
    //     assert_eq!(1, 1);
    // }
}
