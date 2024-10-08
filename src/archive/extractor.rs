use std::fmt::{Display, Formatter};
use std::fs::{File, create_dir, create_dir_all};
use std::io::Read;
use std::path::Path;

use crate::io_utils::byte_buffer::ByteBuffer;
use crate::io_utils::path_utils::{get_superpath, is_a_subdirectory};
use crate::io_utils::{HUFFMAN_SIGNATURE, LZ77_SIGNATURE, bytes_to_u64};

use crate::archive::directory_info::DirectoryInfo;

use crate::compress::Decompress;
use crate::compress::DecompressionError;

use crate::compress::huffman::HuffmanDecompressor;
use crate::compress::lz77::LZ77Decompressor;


pub struct Extractor
{
    decompressor: Box<dyn Decompress>,
    archive_info: Vec<(String, Option<u64>)>,
}

impl Extractor
{
    pub fn new(archive_filename: String) -> Result<Extractor, DecompressionError>
    {
        let mut archive_file = File::open(archive_filename)
            .map_err(|_| DecompressionError::FileOpenError)?;

        // Check signature.
        let huffman_signature: Vec<u8> = HUFFMAN_SIGNATURE.to_be_bytes()
            .into_iter().skip_while(|&byte| byte == 0)
            .collect();

        let lz77_signature: Vec<u8> = LZ77_SIGNATURE.to_be_bytes()
            .into_iter().skip_while(|&byte| byte == 0)
            .collect();


        let mut signature: Vec<u8> = vec![0; huffman_signature.len()];
        archive_file.read(&mut signature)
            .map_err(|_| DecompressionError::BadFormat)?;


        let mut decompressor: Box<dyn Decompress> = if signature == huffman_signature
        {
            Box::new(HuffmanDecompressor::new(archive_file)?)
        }
        else if signature == lz77_signature
        {
            Box::new(LZ77Decompressor::new(archive_file)?)
        }
        else
        {
            return Err(DecompressionError::BadFormat);
        };


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

    pub fn extract_paths(&mut self, paths_to_extract: Vec<String>, output_directory: String)
        -> Result<(), DecompressionError>
    {
        create_dir_all(&output_directory)
            .map_err(|_| DecompressionError::Other)?;

        for (path, size) in &self.archive_info
        {
            // Check if this path is a subdirectory of some given path to be extracted.
            match paths_to_extract.iter()
                .find(|&path_to_extract| is_a_subdirectory(path_to_extract, path))
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
                    let superpath_to_be_stripped = get_superpath(path_to_extract);

                    let path_stripped = path.strip_prefix(&superpath_to_be_stripped)
                        .expect("Bad path stripping.")
                        .to_string();
                    let output_path = format!("{}/{}", output_directory, path_stripped);

                    if Path::new(&output_path).exists()
                    {
                        continue;
                    }

                    match size
                    {
                        None => create_dir(&output_path)    // directory
                            .map_err(|_| DecompressionError::Other)?,

                        Some(bytes) =>                     // regular file
                            self.decompressor.decompress_bytes_to_file(&output_path, *bytes as usize)?,
                    };
                }
            }
        }

        Ok(())
    }
}

impl Display for Extractor
{
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result
    {
        let content = self.get_archive_info()
            .iter()
            .map(|(path, size)| format!("{} {:?}", path, size))
            .collect::<Vec<String>>()
            .join("\n");

        write!(formatter, "{}", content)
    }
}
