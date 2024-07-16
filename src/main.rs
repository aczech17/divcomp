mod config;

use std::env::args;
use config::*;

mod compress_stage;
use compress_stage::compress::compress;
use compress_stage::decompress::decompress;
use crate::archive_stage::archive::archive;
use crate::archive_stage::extract::extract;

mod archive_stage;
mod io_utils;

extern crate colored;

fn main()
{
    let config = match parse_arguments()
    {
        Ok(c) => c,
        Err(err_msg) =>
        {
            eprintln!("{}", err_msg);
            return;
        }
    };

    let result = match config.option
    {
        ConfigOption::Archive =>
            archive(config.input_filenames, config.output_archive_filename.unwrap()),

        ConfigOption::Extract =>
            {
                let archive_filename = &config.input_filenames[0];
                extract(archive_filename)
            }
    };

    if let Err(err_msg) = result
    {
        eprintln!("{}", err_msg);
    }
}

