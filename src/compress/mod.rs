use std::fs;
use std::path::Path;
use crate::archive::archive::archive;
use crate::compress::lz77::LZ77Compressor;
use crate::io_utils::{byte_writer, get_tmp_file_name};

pub mod lz77;
pub mod huffman;
use crate::compress::huffman::HuffmanCompressor;

#[allow(dead_code)]
pub enum CompressionMethod
{
    HUFFMAN, LZ77,
}

#[derive(Debug)]
pub enum DecompressError
{
    EmptyFile, BadFormat, FileTooShort, FileOpenError, Other,
}

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

    let tmp_file_name = get_tmp_file_name()
        .map_err(|_| "Could not find a proper name for a temporary file while archiving.")?;
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


pub fn decompress_error_to_string(error: DecompressError) -> String
{
    match error
    {
        DecompressError::BadFormat | DecompressError::EmptyFile | DecompressError::FileTooShort
        => "Nieprawidłowy plik z archiwum.",
        DecompressError::FileOpenError => "Nie udało się otworzyć pliku.",
        DecompressError::Other => "Błąd dekompresji.",
    }.to_string()
}
