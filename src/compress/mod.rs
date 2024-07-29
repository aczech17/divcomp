use crate::compress::decompress::DecompressError;
use crate::io_utils::byte_writer;
use crate::io_utils::universal_reader;

mod huffman_tree;
pub mod compress;

pub mod decompress;

pub mod huffman_decompressor;
pub mod lz77_decompressor;

pub trait Compress
{
    fn compress(&self, input_filename: &str, output_filename: &str) -> Result<(), String>;
}

pub trait Decompress
{
    fn decompress_bytes_to_memory(&mut self, bytes_to_get: usize)
        -> Result<Vec<u8>, DecompressError>;
    fn decompress_bytes_to_file(&mut self, output_filename: &str, count: usize)
        -> Result<(), DecompressError>;
    fn ignore(&mut self, bytes_count: usize) -> Result<(), DecompressError>;
}

