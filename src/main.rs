mod config;

use std::path::Path;
use rand::Rng;
use config::*;

mod compress_stage;
use compress_stage::compress::compress;
use compress_stage::decompress::decompress;
use crate::archive_stage::archive::archive;
use crate::archive_stage::extract::extract;

mod archive_stage;
mod io_utils;

extern crate colored;

fn get_tmp_file_name() -> Result<String, ()>
{
    const FILENAME_SIZE: usize = 10;
    const MAX_ATTEMPTS_COUNT: usize = 10;

    let mut rng = rand::thread_rng();

    for _ in 0..MAX_ATTEMPTS_COUNT
    {
        let filename: String = (0..FILENAME_SIZE)
            .map(|_| rng.sample(rand::distributions::Alphanumeric))
            .map(char::from)
            .collect();

        if !Path::new(&filename).exists()
        {
            return Ok(filename);
        }
    }

    Err(())
}

fn archive_and_compress(input_paths: Vec<String>, archive_filename: String) -> Result<(), String>
{
    let tmp_file_name = get_tmp_file_name()
        .map_err(|_| "Could not find a proper name for a temporary file while archiving.")?;
    archive(input_paths, tmp_file_name.clone())?;

    compress(&tmp_file_name, &archive_filename)
}

fn decompress_and_extract(archive_filename: String) -> Result<(), String>
{
    let tmp_filename = get_tmp_file_name()
        .map_err(|_| "Could not find a proper name for a temporary file while decompressing.")?;
    decompress(&archive_filename, &tmp_filename)?;

    extract(&tmp_filename)
}

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

