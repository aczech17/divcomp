use std::fs::File;
use crate::compress::decompress::{DecompressError, Dictionary};
use crate::compress::huffman_tree::HuffmanTree;
use crate::io_utils::bit_vector::BitVector;
use crate::io_utils::universal_reader::UniversalReader;
use crate::compress::byte_writer::ByteWriter;
use crate::compress::Decompress;

pub struct HuffmanDecompressor
{
    file_reader: UniversalReader,
    dictionary: Dictionary,
}

impl HuffmanDecompressor
{
    pub fn new(input_file: File) -> Result<HuffmanDecompressor, DecompressError>
    {
        let input_file_size = input_file.metadata()
            .unwrap()
            .len() as usize;

        if input_file_size == 0
        {
            return Err(DecompressError::EmptyFile);
        }

        let mut file_reader = UniversalReader::new(input_file);


        let huffman_tree = HuffmanTree::from_code(&mut file_reader)
            .map_err(|_| DecompressError::BadFormat)?;
        let dictionary = huffman_tree.get_bytes_encoding();

        let decompressor = HuffmanDecompressor
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


    fn decompress_somewhere
    (
        &mut self,
        bytes_count: usize,
        output_filename: Option<String>,
        save_to_memory: bool
    )
        -> Result<Option<Vec<u8>>, DecompressError>
    {

        let mut bytes_decompressed = 0;
        let mut potential_result_vector: Option<Vec<u8>> = match save_to_memory
        {
            true => Some(Vec::with_capacity(bytes_count)),
            false => None,
        };

        let mut potential_file_writer = match output_filename
        {
            Some(filename) =>
                {
                    let writer = ByteWriter::new(&filename)
                        .map_err(|_| DecompressError::Other)?;

                    Some(writer)
                }

            None => None,
        };

        let mut potential_codeword = BitVector::new();
        while bytes_decompressed < bytes_count
        {
            let bit = self.file_reader.read_bit()
                .ok_or(DecompressError::FileTooShort)?;
            potential_codeword.push_bit(bit);

            if let Some(byte) = self.get_byte_from_codeword(&potential_codeword)
            {
                if let Some(vector) = &mut potential_result_vector
                {
                    vector.push(byte);
                }

                if let Some(writer) = &mut potential_file_writer
                {
                    writer.write_byte(byte);
                }

                bytes_decompressed += 1;
                potential_codeword.clear();
            }
        }

        Ok(potential_result_vector)
    }
}

impl Decompress for HuffmanDecompressor
{
    fn decompress_bytes_to_memory(&mut self, bytes_to_get: usize) -> Result<Vec<u8>, DecompressError>
    {
        let bytes =
            self.decompress_somewhere(bytes_to_get, None, true)?;

        Ok(bytes.unwrap())
    }

    fn decompress_bytes_to_file(&mut self, output_filename: &str, count: usize) -> Result<(), DecompressError>
    {
        self.decompress_somewhere(count, Some(output_filename.to_owned()), false)?;

        Ok(())
    }

    fn ignore(&mut self, bytes_count: usize) -> Result<(), DecompressError>
    {
        self.decompress_somewhere(bytes_count, None, false)?;

        Ok(())
    }
}
