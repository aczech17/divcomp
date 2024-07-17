use std::fs;

use crate::archive_stage::archive::archive;
use crate::compress_stage::compress::compress;
use crate::io_utils::get_tmp_file_name;
use crate::main_module::extractor::Extractor;

pub mod config;
pub mod extractor;

pub fn archive_and_compress(input_paths: Vec<String>, archive_filename: String) -> Result<(), String>
{
    let tmp_file_name = get_tmp_file_name()
        .map_err(|_| "Could not find a proper name for a temporary file while archiving.")?;
    archive(input_paths, tmp_file_name.clone())?;

    let compress_result = compress(&tmp_file_name, &archive_filename);

    fs::remove_file(&tmp_file_name)
        .map_err(|_| format!("Could not remove the temporary file {}.", tmp_file_name))?;

    compress_result
}

pub fn print_archive_info(extractor: &Extractor)
{
    for (path, size) in extractor.get_archive_info()
    {
        println!("{} - {:?}", path, size);
    }
}
