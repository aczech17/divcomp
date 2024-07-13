mod huffman_tree;
mod file_reader;
mod bit_vector;
mod compress;
mod file_writer;

extern crate colored;
use colored::*;
use divcomp::compress::compress;
use divcomp::decompress::decompress;

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

