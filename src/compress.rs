use std::fs::{File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use crate::file_writer::FileWriter;
use crate::huffman_tree::HuffmanTree;

fn write_padding_size(output_filename: &str, padding_size: usize, padding_size_position: usize)
{
    let mut file = OpenOptions::new()
        .read(true)
        .append(false)
        .create(false)
        .write(true)
        .open(output_filename)
        .unwrap();

    let byte_position = padding_size_position / 8;
    let bit_position = padding_size_position % 8;

    file.seek(SeekFrom::Start(byte_position as u64))
        .unwrap();
    let mut buffer = [0];
    file.read(&mut buffer)
        .unwrap();

    buffer[0] |= (padding_size as u8) << (7 - bit_position);

    file.seek(SeekFrom::Start(byte_position as u64))
        .unwrap();
    file.write(&buffer)
        .unwrap();
}

pub fn compress(input_filename: &str, output_filename: &str) -> Result<(), String>
{
    let input = match File::open(input_filename)
    {
        Ok(file) => file,
        Err(_) => return Err(format!("Could not open file {}.", input_filename)),
    };

    let huffman_tree = HuffmanTree::new(input);
    if huffman_tree.empty()
    {
        let _empty_file = File::create(output_filename)
            .map_err(|_| format!("Could not create empty file {}.", output_filename))?;

        return Ok(());
    }

    let tree_encoding = huffman_tree.get_tree_encoding();
    let bytes_encoding = huffman_tree.get_bytes_encoding();

    let mut file_writer = match FileWriter::new(output_filename)
    {
        Some(fw) => fw,
        None => return Err(format!("Could not create file writer for {}.", output_filename)),
    };

    file_writer.write_bit_vector(&tree_encoding);
    let padding_size_position = tree_encoding.size();

    // for future padding [0 bits; 7 bits]
    file_writer.write_bit(0);
    file_writer.write_bit(0);
    file_writer.write_bit(0);

    let mut input = match File::open(input_filename)
    {
        Ok(file) => file,
        Err(_) => return Err(format!("Could not open file {} second time.", input_filename)),
    };


    // read byte by byte
    let mut buffer = [0; 1];
    loop
    {
        let bytes_read = input.read(&mut buffer)
            .unwrap();

        if bytes_read == 0
        {
            break;
        }

        let byte = buffer[0];
        let codeword = bytes_encoding.get(&byte)
            .expect(&format!("Could not find codeword for byte {:X}", byte));

        file_writer.write_bit_vector(codeword);
    }

    let padding_size = match file_writer.bits_in_buffer_count() % 8
    {
        0 => 0,
        v => 8 - v,
    };

    file_writer.dump_buffer();
    write_padding_size(output_filename, padding_size, padding_size_position);

    Ok(())
}
