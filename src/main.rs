mod main_module;
mod compress_stage;
mod archive_stage;
mod io_utils;

use main_module::{archive_and_compress, decompress_and_extract};
use main_module::config::{ConfigOption, parse_arguments};


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
            archive_and_compress(config.input_filenames, config.output_archive_filename.unwrap()),

        ConfigOption::Extract =>
        {
            let archive_filename = config.input_filenames[0].clone();
            decompress_and_extract(archive_filename)
        }
    };

    if let Err(err_msg) = result
    {
        eprintln!("{}", err_msg);
    }
}

