use crate::io_utils::FileInfo;
use crate::archive::pack::pack;
use crate::compress::lz77::LZ77Compressor;
use crate::io_utils::byte_writer;
use std::fs;
use std::path::Path;

pub mod lz77;
pub mod huffman;
use crate::compress::huffman::HuffmanCompressor;
use crate::io_utils::create_tmp_file;

#[allow(clippy::upper_case_acronyms)] // Clippy thinks HUFFMAN is an acronym.
#[derive(Clone, Copy, PartialEq)]
pub enum CompressionMethod
{
    HUFFMAN, LZ77,
}

#[derive(Debug)]
pub enum DecompressionError
{
    EmptyFile, BadFormat, FileTooShort, FileOpenError, FileCreationError, Other,
}

pub trait Compress
{
    fn compress(&self, input_filename: &str, output_filename: &str) -> Result<(), String>;
}

pub trait Decompress
{
    fn decompress_bytes_to_memory(&mut self, bytes_to_get: usize)
        -> Result<Vec<u8>, DecompressionError>;
    fn decompress_bytes_to_file(&mut self, output_filename: &str, count: usize)
        -> Result<(), DecompressionError>;
    fn ignore(&mut self, bytes_count: usize) -> Result<(), DecompressionError>;
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


pub fn decompress_error_to_string(error: DecompressionError) -> String
{
    match error
    {
        DecompressionError::BadFormat | DecompressionError::EmptyFile | DecompressionError::FileTooShort
        => "Nieprawidłowy plik z archiwum.",
        DecompressionError::FileOpenError => "Nie udało się otworzyć pliku.",
        DecompressionError::FileCreationError => "Nie udało się utworzyć pliku.",
        DecompressionError::Other => "Błąd dekompresji.",
    }.to_string()
}
