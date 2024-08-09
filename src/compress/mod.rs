use std::fs;
use std::path::Path;
use crate::archive::archive::archive;
use crate::compress::lz77::LZ77Compressor;
use crate::io_utils::byte_writer;

pub mod lz77;
pub mod huffman;
use crate::compress::huffman::HuffmanCompressor;
use crate::io_utils::path_utils::get_tmp_file_path;

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

pub fn archive_and_compress
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

    let tmp_file_name = get_tmp_file_path(".unarch")
        .ok_or("Could not find a proper name for a temporary file while archiving.")?;
    archive(input_paths, tmp_file_name.clone())?;


    let compressor: Box<dyn Compress> = match compression_method
    {
        CompressionMethod::HUFFMAN => Box::new(HuffmanCompressor),
        CompressionMethod::LZ77  => Box::new(LZ77Compressor),
    };
    let compress_result = compressor.compress(&tmp_file_name, &archive_filename);

    fs::remove_file(&tmp_file_name)
        .map_err(|_| format!("Could not remove the temporary file {}.", tmp_file_name))?;

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
