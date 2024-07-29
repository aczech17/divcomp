use std::fs::{create_dir, create_dir_all, File};
use std::io;
use std::io::{Read, Write};
use std::path::Path;

use crate::archive::directory_info::DirectoryInfo;
use crate::compress::Decompress;
use crate::compress::decompress::{DecompressError, HuffmanDecompressor};
use crate::io_utils::byte_buffer::ByteBuffer;
use crate::io_utils::{bytes_to_u64, SIGNATURE};
use crate::io_utils::path_utils::{get_superpath, is_a_subdirectory};

pub struct Extractor
{
    decompressor: HuffmanDecompressor,
    archive_info: Vec<(String, Option<u64>)>,
}

impl Extractor
{
    pub fn new(archive_filename: String) -> Result<Extractor, DecompressError>
    {
        let mut archive_file = File::open(archive_filename)
            .map_err(|_| DecompressError::FileOpenError)?;

        // Check signature.
        let expected_signature: Vec<u8> = SIGNATURE.to_be_bytes().to_vec()
            .into_iter().skip_while(|&byte| byte == 0)
            .collect();

        let mut signature: Vec<u8> = vec![0; expected_signature.len()];
        archive_file.read(&mut signature)
            .map_err(|_| DecompressError::FileTooShort)?;

        if signature != expected_signature
        {
            return Err(DecompressError::BadFormat);
        }


        let mut decompressor = HuffmanDecompressor::new(archive_file)?;

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
        create_dir_all(&output_directory)
            .map_err(|_| DecompressError::Other)?;

        for (path, size) in &self.archive_info
        {
            let output_path = match output_directory.as_str()
            {
                "" => path.to_string(),
                directory => format!("{}/{}", directory, path),
            };

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

    pub fn extract_paths(&mut self, paths_to_extract: Vec<String>, output_directory: String)
        -> Result<(), DecompressError>
    {
        create_dir_all(&output_directory)
            .map_err(|_| DecompressError::Other)?;

        for (path, size) in &self.archive_info
        {
            // Check if this path is a subdirectory of some given path to be extracted.
            match paths_to_extract.iter()
                .find(|&path_to_extract| is_a_subdirectory(path_to_extract, &path))
            {
                None => // This path is not to be extracted. Ignore and continue.
                {
                    if let Some(bytes) = size
                    {
                        self.decompressor.ignore(*bytes as usize)?;
                    }
                    continue;
                }

                Some(path_to_extract) => // This path is a path to be extracted.
                {
                    let superpath_to_be_stripped = get_superpath(&path_to_extract);

                    let path_stripped = path.strip_prefix(&superpath_to_be_stripped)
                        .expect("Bad path stripping.")
                        .to_string();
                    let output_path = match output_directory.as_str()
                    {
                        "" => path_stripped,    // this folder
                        directory => format!("{}/{}", directory, path_stripped),
                    };

                    if Path::new(&output_path).exists()
                    {
                        println!("Path {} exists. Skipping.", path_to_extract);
                        io::stdout().flush().unwrap();

                        continue;
                    }

                    match size
                    {
                        None => create_dir(&output_path)    // directory
                            .map_err(|_| DecompressError::Other)?,

                        Some(bytes) =>                     // regular file
                            self.decompressor.decompress_bytes_to_file(&output_path, *bytes as usize)?,
                    };
                }
            }
        }

        Ok(())
    }
}
