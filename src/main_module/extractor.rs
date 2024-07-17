use std::fs::create_dir;
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

    pub fn extract(&mut self) -> Result<(), DecompressError>
    {
        for (path, size) in &self.archive_info
        {
            match size
            {
                None => create_dir(path).unwrap(),
                Some(size) => self.decompressor.decompress_some_bytes(&path, *size as usize)?,
            }
        }

        Ok(())
    }
}
