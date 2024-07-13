use std::collections::HashMap;
use std::fs;
use std::fs::File;
use std::io::Write;
use crate::bit_vector::BitVector;
use crate::file_reader::FileReader;
use crate::huffman_tree::HuffmanTree;

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
    }

    let input_file = match File::open(input_filename)
    {
        Ok(f) => f,
        Err(err) => return Err(err.to_string()),
    };

    let mut file_reader = FileReader::new(input_file);

    let huffman_tree = HuffmanTree::from_code(&mut file_reader);


    let padding_size = ((file_reader.read_bit().unwrap() << 3) |
                             (file_reader.read_bit().unwrap() << 2) |
                             file_reader.read_bit().unwrap()) as usize;

    let dictionary = huffman_tree.get_bytes_encoding();
    let bits_to_read = input_file_size * 8 - padding_size;

    let mut output_file = match File::create(output_filename)
    {
        Ok(f) => f,
        Err(err) => return Err(err.to_string()),
    };

    let mut buffer = [0];
    let mut potential_codeword = BitVector::new();
    while file_reader.bits_read() < bits_to_read
    {
        let bit = file_reader.read_bit()
            .expect("Unexpected end of file ¯\\(ツ)/¯");

        potential_codeword.push_bit(bit);
        if let Some(byte) = get_byte_from_codeword(&dictionary, &potential_codeword)
        {
            buffer[0] = byte;

            output_file.write(&buffer)
                .unwrap();
            potential_codeword.clear();
        }
    }

    Ok(())
}
