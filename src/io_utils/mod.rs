use std::path::Path;
use rand::Rng;
use sysinfo::System;

pub mod byte_writer;
pub mod universal_reader;
pub mod byte_buffer;
pub mod bit_vector;
pub mod bit_vector_writer;
pub mod path_utils;

pub fn bytes_to_u64(bytes: Vec<u8>) -> u64
{
    let buffer: [u8; 8] = bytes.try_into().unwrap();
    u64::from_be_bytes(buffer)
}

pub fn get_tmp_file_name() -> Result<String, ()>
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

pub fn get_memory_buffers_size() -> usize
{
    let mut system_info = System::new_all();
    system_info.refresh_all();

    let total_memory = system_info.total_memory();
    (total_memory / 16) as usize
}
