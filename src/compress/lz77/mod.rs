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
    input: UniversalReader,
}

impl LZ77Decompressor
{
    pub fn new(input_file: File) -> Result<Self, DecompressionError>
    {
        let decompressor = LZ77Decompressor
        {
            input: UniversalReader::new(input_file),
        };

        Ok(decompressor)
    }

    fn load_u16(&mut self) -> Result<u16, DecompressionError>
    {
        let b1 = self.input.read_byte()
            .ok_or(DecompressionError::Other)?;
        let b2 = self.input.read_byte()
            .ok_or(DecompressionError::Other)?;

        let value = ((b1 as u16) << 8) | (b2 as u16);
        Ok(value)
    }

    fn decompress_bytes_somewhere
    (
        &mut self,
        output_filename: Option<&str>,
        save_to_memory: bool,
        bytes_count: usize
    )
        -> Result<Option<Vec<u8>>, DecompressionError>
    {
        let mut bytes_decompressed = 0;
        let mut decompression_buffer =
            DecompressionBuffer::new(output_filename)?;

        while bytes_decompressed < bytes_count
        {
            let offset = self.load_u16()? as usize;
            let length = self.load_u16()? as usize;

            bytes_decompressed += decompression_buffer.decompress_couple(offset, length)
                .map_err(|_| DecompressionError::Other)?;

            if bytes_decompressed < bytes_count
            {
                if let Some(byte_after) = self.input.read_byte()
                {
                    decompression_buffer.push_byte(byte_after)?;
                    bytes_decompressed += 1;
                }
            }
        }

        let result = match save_to_memory
        {
            true => Some(decompression_buffer.get_data().clone()),
            false => None,
        };

        Ok(result)
    }
}

impl Decompress for LZ77Decompressor
{
    fn decompress_bytes_to_memory(&mut self, bytes_to_get: usize)
        -> Result<Vec<u8>, DecompressionError>
    {
        let bytes =
            self.decompress_bytes_somewhere(None, true, bytes_to_get)?;

        Ok(bytes.unwrap())
    }

    fn decompress_bytes_to_file(&mut self, output_filename: &str, count: usize)
        -> Result<(), DecompressionError>
    {
        self.decompress_bytes_somewhere(Some(output_filename), false, count)?;

        Ok(())
    }

    fn ignore(&mut self, bytes_count: usize) -> Result<(), DecompressionError>
    {
        self.decompress_bytes_somewhere(None, false, bytes_count)?;

        Ok(())
    }
}

//
// #[cfg(test)]
// mod compress_test
// {
//     use crate::compress::Compress;
//     use crate::compress::lz77::LZ77Compressor;
//
//     #[test]
//     fn aaa()
//     {
//         let mut compressor = LZ77Compressor;
//         compressor.compress("test/aaa.txt", "aaa.bin").unwrap();
//     }
//
//     #[test]
//     fn spoko()
//     {
//         let mut compressor = LZ77Compressor;
//         compressor.compress("test/KOKOKOKOEUROSPOKO.txt", "spoko.bin").unwrap();
//     }
//
//     #[test]
//     fn tadeusz()
//     {
//         let mut compressor = LZ77Compressor;
//         compressor.compress("test/pan-tadeusz.txt", "tadek.bin").unwrap();
//     }
//
//     #[test]
//     fn kutas()
//     {
//         let mut compressor = LZ77Compressor;
//         compressor.compress("test/kutas.txt", "kutas.bin").unwrap();
//     }
// }
//
// #[cfg(test)]
// mod decompress_test
// {
//     use std::fs;
//     use std::fs::File;
//     use std::io::Read;
//     use crate::compress::{Compress, Decompress};
//     use crate::compress::lz77::{LZ77Compressor, LZ77Decompressor};
//
//     #[test]
//     fn aaa()
//     {
//         let mut file = File::open("aaa.bin")
//             .unwrap();
//         let mut buffer = [0; 3];
//         file.read(&mut buffer)
//             .unwrap();
//
//
//         let mut decompressor = LZ77Decompressor::new(file)
//             .unwrap();
//         decompressor.decompress_bytes_to_file("aaa.txt", 16)
//             .unwrap();
//     }
//
//     #[test]
//     fn spoko()
//     {
//         let mut file = File::open("spoko.bin")
//             .unwrap();
//         let mut buffer = [0; 3];
//         file.read(&mut buffer)
//             .unwrap();
//
//         let size = (fs::metadata("spoko.bin").unwrap().len() - 3) as usize;
//
//         let mut decompressor = LZ77Decompressor::new(file)
//             .unwrap();
//         decompressor.decompress_bytes_to_file("spoko.txt", 17)
//             .unwrap();
//     }
//
//     #[test]
//     fn kutas()
//     {
//         let mut file = File::open("kutas.bin")
//             .unwrap();
//         let mut buffer = [0; 3];
//         file.read(&mut buffer)
//             .unwrap();
//
//         let mut decompressor = LZ77Decompressor::new(file)
//             .unwrap();
//         decompressor.decompress_bytes_to_file("kutas.txt", 33)
//             .unwrap();
//     }
//
//     #[test]
//     fn kutas_mem()
//     {
//         let mut file = File::open("kutas.bin")
//             .unwrap();
//         let mut buffer = [0; 3];
//         file.read(&mut buffer)
//             .unwrap();
//
//         let mut decompressor = LZ77Decompressor::new(file)
//             .unwrap();
//         let bytes = decompressor.decompress_bytes_to_memory( 33)
//             .unwrap();
//
//         let s = String::from_utf8(bytes)
//             .unwrap();
//         println!("{}", s);
//     }
// }
//
// #[cfg(test)]
// mod integration_test
// {
//     use std::fs;
//     use std::fs::File;
//     use std::io::Read;
//     use crate::compress::{Compress, Decompress};
//     use crate::compress::lz77::{LZ77Compressor, LZ77Decompressor};
//     use crate::io_utils::get_tmp_file_name;
//
//     fn do_test(filename: &str)
//     {
//         let input_file_size = fs::metadata(filename)
//             .unwrap().len() as usize;
//
//         let compressed_file_name = get_tmp_file_name()
//             .unwrap();
//
//         let compressor = LZ77Compressor;
//         compressor.compress(filename, &compressed_file_name)
//             .unwrap();
//         let compressed_file_size = fs::metadata(&compressed_file_name).unwrap()
//             .len() as usize;
//
//
//         let decompressed_file_name = get_tmp_file_name()
//             .unwrap();
//         let mut input_file = File::open(filename)
//             .unwrap();
//
//         let mut signature = [0; 3];
//         input_file.read(&mut signature)
//             .unwrap();
//
//         let mut decompressor = LZ77Decompressor::new(input_file)
//             .unwrap();
//         decompressor.decompress_bytes_to_file(&decompressed_file_name, input_file_size)
//             .unwrap();
//
//         let mut input_file = File::open(filename)
//             .unwrap();
//         let file_size = input_file.metadata().unwrap().len() as usize;
//         let mut decompressed_file = File::open(&decompressed_file_name)
//             .unwrap();
//
//         let mut input_file_content: Vec<u8> = Vec::with_capacity(file_size);
//         let mut decompressed_file_content: Vec<u8> = Vec::with_capacity(file_size);
//
//         input_file.read(&mut input_file_content)
//             .unwrap();
//         decompressed_file.read(&mut decompressed_file_content)
//             .unwrap();
//
//         let files_are_equal = input_file_content == decompressed_file_content;
//
//         fs::remove_file(&compressed_file_name)
//             .unwrap();
//         fs::remove_file(&decompressed_file_name)
//             .unwrap();
//
//         assert!(files_are_equal);
//     }
//
//     #[test]
//     fn multiple_as()
//     {
//         do_test("test/aaa.txt");
//     }
//
//     #[test]
//     fn KOKOKOKO()
//     {
//         do_test("test/KOKOKOKO.txt");
//     }
//
//     #[test]
//     fn KOKOKOKOEUROSPOKO()
//     {
//         do_test("test/KOKOKOKOEUROSPOKO.txt")
//     }
//
//     #[test]
//     fn KOKOKOKOEE()
//     {
//         do_test("test/KOKOKOKOEE.txt");
//     }
//
//     #[test]
//     fn KOKOKOKOEEE()
//     {
//         do_test("test/KOKOKOKOEEE.txt");
//     }
//
//     // #[test]
//     // fn test1()
//     // {
//     //     let compressor = LZ77Compressor;
//     //     compressor.compress("test1.txt", "output.bin").
//     //         unwrap();
//     //
//     //     assert_eq!(1, 1);
//     // }
// }
