use rand::Rng;
use std::env;
use std::fs::File;
use sysinfo::System;

pub mod byte_writer;
pub mod universal_reader;
pub mod byte_buffer;
pub mod bit_vector;
pub mod bit_vector_writer;
pub mod path_utils;

pub const HUFFMAN_SIGNATURE: u64 = 0xAEFE48;
pub const LZ77_SIGNATURE: u64 = 0xAEFE77;

pub struct FileInfo
{
    pub handle: File,
    pub path: String,
}

pub fn bytes_to_u64(bytes: Vec<u8>) -> u64
{
    let buffer: [u8; 8] = bytes.try_into().unwrap();
    u64::from_be_bytes(buffer)
}

pub fn get_memory_buffers_size() -> usize
{
    let mut system_info = System::new_all();
    system_info.refresh_all();

    let total_memory = system_info.total_memory() as usize;
    total_memory / 16
}

pub fn create_tmp_file(extension: &str) -> Option<FileInfo>
{
    let tmp_directory = if cfg!(unix)
    {
        "/tmp".to_string()
    }
    else
    {
        env::var("TEMP").unwrap_or_else(|_| String::from("."))
    };

    const FILENAME_SIZE: usize = 10;
    const MAX_ATTEMPTS_COUNT: usize = 10;

    let mut rng = rand::thread_rng();

    for _ in 0..MAX_ATTEMPTS_COUNT
    {
        let filename: String = (0..FILENAME_SIZE)
            .map(|_| rng.sample(rand::distr::Alphanumeric))
            .map(char::from)
            .collect();
        let path = format!("{tmp_directory}/{filename}{extension}");

        if let Ok(file) = File::create(&path)
        {
            let file_info = FileInfo
            {
                handle: file,
                path,
            };

            return Some(file_info);
        }
    }

    None
}
