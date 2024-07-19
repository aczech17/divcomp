use std::fs::create_dir;
use std::io;
use std::io::Write;
use std::path::Path;
use crate::archive_stage::directory_info::DirectoryInfo;
use crate::compress_stage::decompress::{DecompressError, Decompressor};
use crate::io_utils::byte_buffer::ByteBuffer;
use crate::io_utils::bytes_to_u64;

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

    pub fn extract_all(&mut self) -> Result<(), DecompressError>
    {
        for (path, size) in &self.archive_info
        {
            if Path::new(path).exists()
            {
                print!("{} already exists, skipping...", path);
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
                None => create_dir(path).map_err(|_| DecompressError::Other)?,
                Some(size) =>
                    self.decompressor.decompress_bytes_to_file(&path, *size as usize)?,
            }

            println!("extracted.");
        }

        Ok(())
    }

    fn is_a_subdirectory(superpath: &str, subpath: &str) -> bool
    {
        // First define the slash kind in the OS.
        let slash = match superpath.find("/")
        {
            Some(_) => "/",
            None => "\\",
        };

        let superdirectories: Vec<&str> = superpath.split(slash)
            .collect();
        let subdirectories: Vec<&str> = subpath.split(slash)
            .collect();

        if superdirectories.len() > subdirectories.len()
        {
            return false;
        }

        for (i, elem) in superdirectories.iter().enumerate()
        {
            if &subdirectories[i] != elem
            {
                return false;
            }
        }
        true
    }

    pub fn extract_path(&mut self, path_to_extract: String) -> Result<(), DecompressError>
    {
        let mut path_reached = false;

        for (path, size) in &self.archive_info
        {
            let is_subdirectory = Self::is_a_subdirectory(&path_to_extract, path);
            if !is_subdirectory && !path_reached
            {
                // It's not this yet.

                // If it's not a directory, read the file content and ignore it.
                if let Some(bytes) = size
                {
                    self.decompressor.ignore(*bytes as usize)?;
                }

                continue;
            }

            if !is_subdirectory && path_reached
            {
                // What had to be extracted, was extracted. It's finished.
                break;
            }

            if is_subdirectory
            {
                path_reached = true;

                match size
                {
                    None => create_dir(path).map_err(|_| DecompressError::Other)?,
                    Some(size) =>
                        self.decompressor.decompress_bytes_to_file(&path, *size as usize)?,
                }
            }
        }

        Ok(())
    }
}
