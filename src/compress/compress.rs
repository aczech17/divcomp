use std::fs;
use std::fs::File;
use std::path::Path;
use crate::archive::archive::archive;
use crate::compress::huffman_tree::HuffmanTree;
use crate::io_utils::bit_vector::BitVector;
use crate::io_utils::bit_vector_writer::BitVectorWriter;
use crate::io_utils::{get_tmp_file_name, SIGNATURE};
use crate::io_utils::universal_reader::UniversalReader;

fn compress(input_filename: &str, output_filename: &str) -> Result<(), String>
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

    let mut file_writer = match BitVectorWriter::new(output_filename)
    {
        Some(fw) => fw,
        None => return Err(format!("Could not create file writer for {}.", output_filename)),
    };

    // Start writing to file.
    file_writer.write_bit_vector(&BitVector::from_u64(SIGNATURE));
    file_writer.write_bit_vector(&tree_encoding);


    // Reopen the file.
    let input = match File::open(input_filename)
    {
        Ok(file) => file,
        Err(_) => return Err(format!("Could not open file {} second time.", input_filename)),
    };
    let mut buffer = UniversalReader::new(input);


    // read byte by byte
    while let Some(byte) = buffer.read_byte()
    {
        let codeword = bytes_encoding.get(&byte)
            .ok_or(&format!("Could not find codeword for byte {:X}", byte))?;

        file_writer.write_bit_vector(codeword);
    }

    Ok(())
}

pub fn archive_and_compress(input_paths: Vec<String>, archive_filename: String) -> Result<(), String>
{
    if Path::new(&archive_filename).exists()
    {
        return Err("Path already exists.".to_string());
    }

    let tmp_file_name = get_tmp_file_name()
        .map_err(|_| "Could not find a proper name for a temporary file while archiving.")?;
    archive(input_paths, tmp_file_name.clone())?;

    let compress_result = compress(&tmp_file_name, &archive_filename);

    fs::remove_file(&tmp_file_name)
        .map_err(|_| format!("Could not remove the temporary file {}.", tmp_file_name))?;

    compress_result
}

