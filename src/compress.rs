use std::fs::{File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use crate::bit_vector::Bit;
use crate::file_writer::FileWriter;
use crate::huffman_tree::HuffmanTree;

fn write_bit_to_file(file: &mut File, bit: Bit, bit_position: usize)
{
    let byte_position = bit_position / 8;
    let position_in_bit = bit_position % 8;

    file.seek(SeekFrom::Start(byte_position as u64))
        .unwrap();

    let mut buffer = [0];
    file.read(&mut buffer)
        .unwrap();

    buffer[0] |= bit << (7 - position_in_bit);

    file.seek(SeekFrom::Start(byte_position as u64))
        .unwrap();
    file.write(&buffer)
        .unwrap();
}

fn write_padding_size(output_filename: &str, padding_size: usize, padding_size_position: usize)
{
    let mut file = OpenOptions::new()
        .read(true)
        .append(false)
        .create(false)
        .write(true)
        .open(output_filename)
        .unwrap();

    write_bit_to_file(&mut file, (padding_size >> 2) as Bit, padding_size_position);
    write_bit_to_file(&mut file, (padding_size >> 1) as Bit, padding_size_position + 1);
    write_bit_to_file(&mut file, padding_size as Bit, padding_size_position + 2);
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

    let padding_size = (8 - file_writer.bits_in_buffer_count() % 8) % 8;

    file_writer.dump_buffer();
    write_padding_size(output_filename, padding_size, padding_size_position);

    Ok(())
}

#[cfg(test)]
mod tests
{
    use std::fs;
    use std::fs::File;
    use std::io::{Read, Write};
    use std::sync::atomic::{AtomicUsize, Ordering};
    use crate::compress::compress;

    static TEST_ID_COUNTER: AtomicUsize = AtomicUsize::new(0);

    fn get_unique_filename(base: &str) -> String
    {
        let id = TEST_ID_COUNTER.fetch_add(1, Ordering::Relaxed);
        format!("{}_{}.bin", base, id)
    }

    fn get_output(input_data: Vec<u8>) -> Vec<u8>
    {
        let input_filename = get_unique_filename("input");
        let output_filename = get_unique_filename("output");

        let mut input_file = File::create(&input_filename)
            .expect("Could not create test input file.");

        input_file.write_all(&input_data)
            .unwrap();

        compress(&input_filename, &output_filename)
            .unwrap();

        let mut output_file = File::open(&output_filename)
            .unwrap();

        let mut output_data: Vec<u8> = vec![];
        let output_file_size = output_file.metadata().unwrap().len() as usize;
        output_data.resize(output_file_size, 0);

        output_file.read_exact(&mut output_data)
            .unwrap();

        fs::remove_file(input_filename)
            .unwrap();
        fs::remove_file(output_filename)
            .unwrap();

        output_data
    }

    fn do_test(input: Vec<u8>, expected_output: Vec<u8>)
    {
        let output = get_output(input);
        assert_eq!(output, expected_output);
    }

    #[test]
    fn empty_file()
    {
        let input = vec![];
        let expected_output = vec![];
        do_test(input, expected_output);
    }

    #[test]
    fn six_a_letters()
    {
        let input = vec![0x61, 0x61, 0x61, 0x61, 0x61, 0x61];
        let expected_output = vec![0xB0, 0xE0, 0x00];
        do_test(input, expected_output);
    }
}

