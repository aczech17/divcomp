use std::fs::File;
use crate::compress::decompress::{DecompressError, HuffmanDecompressor};
use crate::io_utils::universal_reader;
use crate::io_utils::byte_writer;
mod huffman_tree;
pub mod compress;

pub mod decompress;

pub trait Decompress
{
    fn decompress_bytes_to_memory(&mut self, bytes_to_get: usize)
        -> Result<Vec<u8>, DecompressError>;
    fn decompress_bytes_to_file(&mut self, output_filename: &str, count: usize)
        -> Result<(), DecompressError>;
    fn ignore(&mut self, bytes_count: usize) -> Result<(), DecompressError>;
}
