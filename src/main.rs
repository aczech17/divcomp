use main_module::archive_and_compress;
use main_module::config::{ConfigOption, parse_arguments};
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

    match config.option
    {
        ConfigOption::Archive =>
            archive_and_compress(config.input_filenames, config.output_archive_filename.unwrap())
            .unwrap(),
        ConfigOption::Extract =>
        {
            let archive_filename = config.input_filenames[0].clone();
            let mut extractor = Extractor::new(archive_filename)
                .unwrap();

            print_archive_info(&extractor);

            extractor.extract()
                .unwrap();
        }
    }

}

