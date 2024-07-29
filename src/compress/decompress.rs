use std::collections::HashMap;
use std::fs::File;

use crate::compress::byte_writer::ByteWriter;
use crate::compress::Decompress;
use crate::compress::huffman_tree::HuffmanTree;
use crate::compress::universal_reader::UniversalReader;
use crate::io_utils::bit_vector::BitVector;

#[derive(Debug)]
pub enum DecompressError
{
    EmptyFile, BadFormat, FileTooShort, FileOpenError, Other,
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

pub type Dictionary = HashMap<u8, BitVector>;
