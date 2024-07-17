pub mod byte_writer;
pub mod universal_reader;
pub mod byte_buffer;
pub mod bit_vector;
pub mod bit_vector_writer;

pub fn bytes_to_u64(bytes: Vec<u8>) -> u64
{
    let buffer: [u8; 8] = bytes.try_into().unwrap();
    u64::from_be_bytes(buffer)
}
