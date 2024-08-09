use sysinfo::System;

pub mod byte_writer;
pub mod universal_reader;
pub mod byte_buffer;
pub mod bit_vector;
pub mod bit_vector_writer;
pub mod path_utils;

pub const HUFFMAN_SIGNATURE: u64 = 0xAEFE48;
pub const LZ77_SIGNATURE: u64 = 0xAEFE77;

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

