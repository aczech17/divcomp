use std::fs;
use std::path::Path;
use std::fmt::Display;
use crate::io_utils::{FileInfo, create_tmp_file, byte_writer};

use crate::archive::pack::pack;

pub mod huffman;
pub mod lz77;

use crate::compress::huffman::HuffmanCompressor;
use crate::compress::lz77::LZ77Compressor;


#[allow(clippy::upper_case_acronyms)] // Clippy thinks HUFFMAN is an acronym.
#[derive(Clone, Copy, PartialEq)]
pub enum CompressionMethod
{
    HUFFMAN, LZ77,
}

pub trait Compress
{
    fn compress(&self, input_filename: &str, output_filename: &str) -> Result<(), String>;
}

pub fn pack_and_compress
(
    input_paths: Vec<String>,
    archive_filename: String,
    compression_method: CompressionMethod
)
    -> Result<(), String>
{
    if Path::new(&archive_filename).exists()
    {
        return Err("Path already exists.".to_string());
    }

    let FileInfo
    {
        handle: tmp_file,
        path: tmp_file_path
    }
        = create_tmp_file(".unarch")
        .ok_or("Could not create a temporary file while archiving.")?;

    pack(input_paths, tmp_file)?;


    let compressor: Box<dyn Compress> = match compression_method
    {
        CompressionMethod::HUFFMAN => Box::new(HuffmanCompressor),
        CompressionMethod::LZ77  => Box::new(LZ77Compressor),
    };
    let compress_result = compressor.compress(&tmp_file_path, &archive_filename);

    fs::remove_file(&tmp_file_path)
        .map_err(|_| format!("Could not remove the temporary file {}.", tmp_file_path))?;

    compress_result
}

pub trait Decompress
{
    fn decompress_bytes_to_memory(&mut self, bytes_to_get: usize)
                                  -> Result<Vec<u8>, DecompressionError>;
    fn decompress_bytes_to_file(&mut self, output_filename: &str, count: usize)
                                -> Result<(), DecompressionError>;
    fn ignore(&mut self, bytes_count: usize) -> Result<(), DecompressionError>;
}

#[derive(Debug)]
pub enum DecompressionError
{
    BadFormat,
    FileOpenError,
    FileCreationError,
    Other,
}

impl Display for DecompressionError
{
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    {
        let message = match self
        {
            DecompressionError::BadFormat           => "Nieprawidłowy plik z archiwum.",
            DecompressionError::FileOpenError       => "Nie udało się otworzyć pliku.",
            DecompressionError::FileCreationError   => "Nie udało się utworzyć pliku.",
            DecompressionError::Other               => "Błąd dekompresji.",
        }.to_string();

        write!(formatter, "{}", message)
    }
}
