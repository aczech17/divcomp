pub mod config;

use std::fs;
use std::path::Path;
use rand::Rng;

use crate::compress_stage::compress::compress;
use crate::compress_stage::decompress::decompress;
use crate::archive_stage::archive::archive;
use crate::archive_stage::extract::extract;

pub const FILE_SIGNATURE: &str = "SAAC";


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

pub fn archive_and_compress(input_paths: Vec<String>, archive_filename: String) -> Result<(), String>
{
    let tmp_file_name = get_tmp_file_name()
        .map_err(|_| "Could not find a proper name for a temporary file while archiving.")?;
    archive(input_paths, tmp_file_name.clone())?;

    let compress_result = compress(&tmp_file_name, &archive_filename);

    fs::remove_file(tmp_file_name)
        .unwrap();

    compress_result
}

pub fn decompress_and_extract(archive_filename: String) -> Result<(), String>
{
    let tmp_filename = get_tmp_file_name()
        .map_err(|_| "Could not find a proper name for a temporary file while decompressing.")?;
    decompress(&archive_filename, &tmp_filename)?;

    let extract_result = extract(&tmp_filename);

    fs::remove_file(tmp_filename)
        .unwrap();

    extract_result
}

