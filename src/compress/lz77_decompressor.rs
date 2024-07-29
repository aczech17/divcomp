use std::fs::File;
use crate::compress::Decompress;
use crate::compress::decompress::DecompressError;

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