use std::collections::HashMap;
use std::fs;
use std::fs::File;
use crate::io_utils::bit_vector::BitVector;
use crate::compress_stage::byte_writer::ByteWriter;
use crate::compress_stage::universal_reader::UniversalReader;
use crate::compress_stage::huffman_tree::HuffmanTree;

use crate::main_module::FILE_SIGNATURE;

fn get_byte_from_codeword(dictionary: &HashMap<u8, BitVector>, potential_codeword: &BitVector) -> Option<u8>
{
    for (byte, value) in dictionary
    {
        if value == potential_codeword
        {
            return Some(*byte);
        }
    }

    None
}

pub fn decompress(input_filename: &str, output_filename: &str) -> Result<(), String>
{
    let input_file_size = fs::metadata(input_filename)
        .unwrap()
        .len() as usize;

    if input_file_size == 0
    {
        let _empty_file = File::create(output_filename).map_err(|s| s.to_string())?;
        return Ok(());
    }

    let input_file = match File::open(input_filename)
    {
        Ok(f) => f,
        Err(err) => return Err(err.to_string()),
    };
    let mut file_reader = UniversalReader::new(input_file);

    // Read the file signature.
    let signature_bytes = file_reader.read_some_bytes(FILE_SIGNATURE.len())
        .map_err(|_| "Could not read file signature.")?;
    let signature = String::from_utf8(signature_bytes)
        .map_err(|_| "Could not parse file signature.")?;
    if signature != FILE_SIGNATURE
    {
        return Err("Bad file format".to_string());
    }


    let huffman_tree = HuffmanTree::from_code(&mut file_reader)
        .map_err(|_| "Could not recreate tree from code.")?;
    let dictionary = huffman_tree.get_bytes_encoding();


    let padding_size = ((file_reader.read_bit().unwrap() << 2) |
                             (file_reader.read_bit().unwrap() << 1) |
                             file_reader.read_bit().unwrap()) as usize;

    let bits_to_read = input_file_size * 8 - padding_size;


    let mut output_writer = ByteWriter::new(output_filename)?;

    let mut potential_codeword = BitVector::new();
    while file_reader.bits_read() < bits_to_read
    {
        let bit = file_reader.read_bit()
            .expect("Unexpected end of file ¯\\(ツ)/¯");

        potential_codeword.push_bit(bit);
        if let Some(byte) = get_byte_from_codeword(&dictionary, &potential_codeword)
        {
            output_writer.write_byte(byte);
            potential_codeword.clear();
        }
    }

    Ok(())
}

