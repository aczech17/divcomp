use crate::compress::compress;

mod huffman_tree;
mod file_reader;
mod bit_vector;
mod compress;
mod file_writer;

fn main()
{
    let input_filename = "Cargo.toml";
    let output_filename = "test.test2";

    compress(input_filename, output_filename)
        .unwrap();
}
