use crate::compress::compress;

mod huffman_tree;
mod file_reader;
mod bit_vector;
mod compress;
mod file_writer;

fn main()
{
    let input_filename = "one.txt";
    let output_filename = "one.bin";

    compress(input_filename, output_filename)
        .unwrap();
}
