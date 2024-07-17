pub mod config;

use std::fs;
use std::path::Path;
use rand::Rng;

use crate::compress_stage::compress::compress;
use crate::archive_stage::archive::archive;
use crate::archive_stage::extract::extract;
use crate::archive_stage::directory_info::DirectoryInfo;
use crate::compress_stage::decompress::{DecompressError, Decompressor};
use crate::io_utils::byte_buffer::ByteBuffer;
use crate::io_utils::bytes_to_u64;

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

struct Extractor
{
    decompressor: Decompressor,
    archive_info: Vec<(String, Option<u64>)>,
}

impl Extractor
{
    pub fn new(archive_filename: String) -> Result<Extractor, DecompressError>
    {
        let mut decompressor = Decompressor::new(&archive_filename)?;

        let header_size =
            bytes_to_u64(decompressor.partial_decompress_to_memory(8)?);

        let header_data = decompressor.partial_decompress_to_memory(header_size as usize)?;
        let mut header_data = ByteBuffer::new(header_data);

        let mut directory_infos = vec![];
        while !header_data.end()
        {
            let directory_info_size = bytes_to_u64(header_data.get_bytes(8)) as usize;
            let directory_info =
                DirectoryInfo::from_bytes(&header_data.get_bytes(directory_info_size));
            directory_infos.push(directory_info);
        }

        let archive_info: Vec<(String, Option<u64>)> = directory_infos.iter()
            .flat_map(|info| info.get_paths_and_sizes())
            .collect();

        let extractor = Extractor
        {
            decompressor,
            archive_info,
        };

        Ok(extractor)
    }

    pub fn get_archive_info(&self) -> &Vec<(String, Option<u64>)>
    {
        &self.archive_info
    }
}


