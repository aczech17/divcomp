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
    decompression_buffer: DecompressionBuffer,
}

impl LZ77Decompressor
{
    pub fn new(input_file: File) -> Result<Self, DecompressionError>
    {
        let decompressor = LZ77Decompressor
        {
            input: UniversalReader::new(input_file),
            decompression_buffer: DecompressionBuffer::new()?,
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

    fn decompress_bytes
    (
        &mut self,
        output_filename: Option<&str>,
        save_to_memory: bool,
        bytes_to_get: usize
    )
        -> Result<Option<Vec<u8>>, DecompressionError>
    {
        self.decompression_buffer.set_output_file(output_filename)?;
        println!("ustawiono output");

        let mut bytes_decompressed = self.decompression_buffer.bytes_in_buffer();
        println!("zdekompresowanych na zapas mamy już {}", bytes_decompressed);

        while bytes_decompressed < bytes_to_get
        {
            let offset = self.load_u16()? as usize;
            let length = self.load_u16()? as usize;
            println!("offset: {}, length: {}", offset, length);

            bytes_decompressed += self.decompression_buffer.decompress_couple(offset, length)?;
            println!("zdekompresowanych: {}", bytes_decompressed);

            if save_to_memory || bytes_decompressed < bytes_to_get
            {
                let byte_after = self.input.read_byte()
                    .ok_or(DecompressionError::Other)?;

                self.decompression_buffer.push_byte(byte_after)?;
                bytes_decompressed += 1;
            }
        }
        println!("zdekompresowanych jest {}", bytes_decompressed);

        let bytes_returned = match save_to_memory
        {
            true => Some(self.decompression_buffer.get_slice_of_data(0..bytes_to_get)),
            false => None,
        };

        // If there are bytes remaining, flush bytes for the current file and keep the rest.
        let rest_count = bytes_decompressed - bytes_to_get;
        let bytes_to_flush = 0..
            self.decompression_buffer.bytes_in_buffer() - rest_count;
        println!("Zostało:");
        for b in self.decompression_buffer.get_data()//[bytes_to_flush.clone()]
        {
            print!("{}", *b as char);
        }
        println!();

        self.decompression_buffer.flush(bytes_to_flush)?;

        println!("Po flushu zostało:");
        for b in self.decompression_buffer.get_data()//[bytes_to_flush.clone()]
        {
            print!("{}", *b as char);
        }
        println!();

        Ok(bytes_returned)
    }
}

impl Decompress for LZ77Decompressor
{
    fn decompress_bytes_to_memory(&mut self, bytes_to_get: usize)
        -> Result<Vec<u8>, DecompressionError>
    {
        let bytes = self.decompress_bytes(None, true, bytes_to_get)?;

        Ok(bytes.unwrap())
    }

    fn decompress_bytes_to_file(&mut self, output_filename: &str, bytes_to_get: usize)
        -> Result<(), DecompressionError>
    {
        self.decompress_bytes(Some(output_filename), false, bytes_to_get)?;
        Ok(())
    }

    fn ignore(&mut self, bytes_count: usize) -> Result<(), DecompressionError>
    {
        self.decompress_bytes(None, false, bytes_count)?;
        Ok(())
    }
}

#[cfg(test)]
mod test
{
    use std::fs::File;
    use std::io::Read;
    use crate::compress::{Compress, Decompress};
    use crate::compress::lz77::{LZ77Compressor, LZ77Decompressor};

    #[test]
    fn test1()
    {
        {
            let compressor = LZ77Compressor;
            compressor.compress("test/papież.txt", "koko.bin").unwrap();
        }

        let mut file = File::open("koko.bin")
            .unwrap();
        let mut signature = [0; 3];
        file.read_exact(&mut signature)
            .unwrap();

        let mut decompressor  = LZ77Decompressor::new(file).unwrap();
        let content = decompressor.decompress_bytes_to_memory(2)
            .unwrap();

        println!("otrzymaliśmy bajty: ");
        for byte in content
        {
            print!("{}", byte as char);
        }
        println!();

        decompressor.decompress_bytes_to_file("file", 5)
            .unwrap();
        decompressor.decompress_bytes_to_file("file2", 3).unwrap();
        decompressor.decompress_bytes_to_file("file3", 2).unwrap();
        decompressor.decompress_bytes_to_file("file4", 2).unwrap();
    }
}
