use main_module::archive_and_compress;
use main_module::config::{ConfigOption, parse_arguments};
use crate::compress_stage::decompress::DecompressError;
use crate::main_module::extractor::Extractor;
use crate::main_module::print_archive_info;
//use crate::main_module

mod main_module;
mod compress_stage;
mod archive_stage;
mod io_utils;

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

    if config.option == ConfigOption::Archive
    {
        match archive_and_compress(config.input_filenames, config.output_archive_filename.unwrap())
        {
            Ok(_) => return,
            Err(err) =>
            {
                eprintln!("{}", err);
                return;
            }
        }
    }

    let archive_filename = config.input_filenames[0].clone();
    let mut extractor = match Extractor::new(archive_filename)
    {
        Ok(e) => e,
        Err(err) =>
            {
                eprintln!("{:?}", err);
                return;
            }
    };

    let result = match config.option
    {
        ConfigOption::Extract => extractor.extract(),
        ConfigOption::Display =>
        {
            print_archive_info(&extractor);
            return;
        }

        _ => Ok(()),
    };

    if let Err(err) = result
    {
        eprintln!("{:?}", err);
    }
}

