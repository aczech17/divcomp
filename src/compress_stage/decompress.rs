use std::collections::HashMap;
use std::fs;
use std::fs::File;

use crate::compress_stage::byte_writer::ByteWriter;
use crate::compress_stage::huffman_tree::HuffmanTree;
use crate::compress_stage::universal_reader::UniversalReader;
use crate::io_utils::bit_vector::BitVector;

#[derive(Debug)]
pub enum DecompressError
{
    EmptyFile, BadFormat, FileTooShort, FileOpenError, Other,
}

type Dictionary = HashMap<u8, BitVector>;

pub struct Decompressor
{
    file_reader: UniversalReader,
    dictionary: Dictionary,
}

impl Decompressor
{

    pub fn new(input_filename: &str) -> Result<Decompressor, DecompressError>
    {
        let input_file_size = fs::metadata(input_filename)
            .unwrap()
            .len() as usize;

        if input_file_size == 0
        {
            return Err(DecompressError::EmptyFile);
        }

        let input_file = match File::open(input_filename)
        {
            Ok(f) => f,
            Err(_) => return Err(DecompressError::FileOpenError)
        };
        let mut file_reader = UniversalReader::new(input_file);


        let huffman_tree = HuffmanTree::from_code(&mut file_reader)
            .map_err(|_| DecompressError::BadFormat)?;
        let dictionary = huffman_tree.get_bytes_encoding();

        let decompressor = Decompressor
        {
            file_reader,
            dictionary,
        };

        Ok(decompressor)
    }

    fn get_byte_from_codeword(&self, potential_codeword: &BitVector) -> Option<u8>
    {
        self.dictionary.iter()
            .find(|&(_, value)| value == potential_codeword)
            .map(|(&byte, _)| byte)
    }


    pub fn decompress_bytes_to_memory(&mut self, bytes_to_get: usize)
        -> Result<Vec<u8>, DecompressError>
    {
        let mut bytes = Vec::with_capacity(bytes_to_get);

        let mut potential_codeword = BitVector::new();
        while bytes.len() < bytes_to_get
        {
            let bit = self.file_reader.read_bit()
                .ok_or(DecompressError::FileTooShort)?;

            potential_codeword.push_bit(bit);
            if let Some(byte) = self.get_byte_from_codeword(&potential_codeword)
            {
                bytes.push(byte);
                potential_codeword.clear();
            }
        }

        Ok(bytes)
    }

    pub fn decompress_bytes_to_file(&mut self, output_filename: &str, count: usize)
        -> Result<(), DecompressError>
    {
        let mut bytes_decompressed = 0;

        let mut output_writer = ByteWriter::new(output_filename)
            .map_err(|_| DecompressError::Other)?;

        let mut potential_codeword = BitVector::new();
        while bytes_decompressed < count
        {
            let bit = self.file_reader.read_bit()
                .ok_or(DecompressError::FileTooShort)?;

            potential_codeword.push_bit(bit);
            if let Some(byte) = self.get_byte_from_codeword(&potential_codeword)
            {
                output_writer.write_byte(byte);
                bytes_decompressed += 1;
                potential_codeword.clear();
            }
        }

        Ok(())
    }

    pub fn ignore(&mut self, bytes_count: usize) -> Result<(), DecompressError>
    {
        let mut bytes_decompressed = 0;
        let mut potential_codeword = BitVector::new();
        while bytes_decompressed < bytes_count
        {
            let bit = self.file_reader.read_bit()
                .ok_or(DecompressError::FileTooShort)?;

            potential_codeword.push_bit(bit);
            if let Some(_byte) = self.get_byte_from_codeword(&potential_codeword)
            {
                bytes_decompressed += 1;
                potential_codeword.clear();
            }
        }

        Ok(())
    }
}
