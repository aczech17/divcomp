pub mod bit_vector;
pub mod file_writer;
pub mod huffman_tree;
pub mod compress;
pub mod file_reader;
pub mod decompress;


extern crate colored;

use compress::compress;
use decompress::decompress;

mod config;
use config::*;


fn main()
{
    let config = match parse_arguments()
    {
        Ok(c) => c,
        Err(err_msg) =>
        {
            eprintln!("{err_msg}");
            return;
        }
    };

    let status = match &config.option
    {
        Option::Compress => compress(&config.input_filename, &config.output_filename),
        Option::Decompress => decompress(&config.input_filename, &config.output_filename),
    };

    if let Err(err_msg) = status
    {
        eprintln!("{err_msg}");
        return;
    }

    print_statistics(config);
}

