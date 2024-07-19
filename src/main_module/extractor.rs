use std::fs::create_dir;
use std::{fs, io};
use std::io::Write;
use std::path::Path;

use crate::archive_stage::directory_info::DirectoryInfo;
use crate::compress_stage::decompress::{DecompressError, Decompressor};
use crate::io_utils::byte_buffer::ByteBuffer;
use crate::io_utils::bytes_to_u64;
use crate::io_utils::path_utils::{get_superpath, is_a_subdirectory};

pub struct Extractor
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
            bytes_to_u64(decompressor.decompress_bytes_to_memory(8)?);

        let header_data = decompressor.decompress_bytes_to_memory(header_size as usize)?;
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

    pub fn extract_all(&mut self, output_directory: String) -> Result<(), DecompressError>
    {
        fs::create_dir_all(&output_directory)
            .map_err(|_| DecompressError::Other)?;

        for (path, size) in &self.archive_info
        {
            let output_path = format!("{}/{}", output_directory, path);
            if Path::new(&output_path).exists()
            {
                print!("{} already exists, skipping...", output_path);
                io::stdout().flush().unwrap();

                if let Some(bytes_count) = size
                {
                    self.decompressor.ignore(*bytes_count as usize)?;
                }
                println!();
                continue;
            }

            print!("{}... ", path);
            io::stdout().flush().unwrap();

            match size
            {
                None => create_dir(output_path).map_err(|_| DecompressError::Other)?,
                Some(size) =>
                    self.decompressor.decompress_bytes_to_file(&output_path, *size as usize)?,
            }

            println!("extracted.");
        }

        Ok(())
    }

    pub fn extract_path(&mut self, path_to_extract: String, output_directory: String)
        -> Result<(), DecompressError>
    {
        fs::create_dir_all(&output_directory)
            .map_err(|_| DecompressError::Other)?;

        // Skip paths that are not subdirectories of the path to be extracted.
        let mut skipped_count = 0;
        for (path, size) in &self.archive_info
        {
            if !is_a_subdirectory(&path_to_extract, path)
            {
                if let Some(bytes) = size
                {
                    self.decompressor.ignore(*bytes as usize)?;
                }
                skipped_count += 1;
            }
        }

        let superpath = get_superpath(&path_to_extract);

        let info = self.archive_info.iter().skip(skipped_count);
        for (path, size) in info
        {
            if !is_a_subdirectory(&path_to_extract, path)
            {
                break;
            }

            let path_stripped = path.strip_prefix(&superpath)
                .expect("Bad path trimming.")
                .to_string();
            let path_to_extract = match output_directory.as_str()
            {
                "" => path_stripped,
                directory => format!("{}/{}", directory, path_stripped),
            };

            if Path::new(&path_to_extract).exists()
            {
                println!("Path {} exists. Skipping.", path_to_extract);
                continue;
            }

            match size
            {
                Some(bytes) =>
                    self.decompressor.decompress_bytes_to_file(&path_to_extract, *bytes as usize)?,
                None => create_dir(&path_to_extract)
                    .map_err(|_| DecompressError::Other)?,
            };
        }

        Ok(())
    }
}
