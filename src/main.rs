mod config;
use config::*;

mod compress_stage;
use compress_stage::compress::compress;
use compress_stage::decompress::decompress;

mod archive_stage;

extern crate colored;

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

