use std::fs::{create_dir, File};
use crate::archive_stage::directory_info::DirectoryInfo;
use crate::io_utils::byte_buffer::ByteBuffer;
use crate::io_utils::byte_writer::ByteWriter;
use crate::io_utils::universal_reader::UniversalReader;

fn bytes_to_u64(bytes: Vec<u8>) -> u64
{
    let buffer: [u8; 8] = bytes.try_into().unwrap();
    u64::from_be_bytes(buffer)
}

fn read_u64(reader: &mut UniversalReader) -> Result<u64, String>
{
    let bytes = reader.read_some_bytes(8)
        .map_err(|_| "Could not read 8 bytes.")?;

    let value = bytes_to_u64(bytes);
    Ok(value)
}

fn extract_file_content(path: String, file_size: u64, reader: &mut UniversalReader)
{
    let mut writer = ByteWriter::new(&path)
        .unwrap();

    for _ in 0..file_size
    {
        let byte = reader.read_byte()
            .unwrap();
        writer.write_byte(byte);
    }
}

pub fn extract(archive_filename: &str) -> Result<(), String>
{
    let archive_file = File::open(archive_filename)
        .map_err(|err| err.to_string())?;
    let mut reader = UniversalReader::new(archive_file);


    let header_size = read_u64(&mut reader)?
                            as usize;

    let header_bytes = reader.read_some_bytes(header_size)
        .map_err(|_| "Could not read header.")?;
    let mut header_bytes = ByteBuffer::new(header_bytes);

    let mut all_dir_infos = vec![];
    while !header_bytes.end()
    {
        let dir_info_size = bytes_to_u64(header_bytes.get_bytes(8)) as usize;

        let dir_info_bytes = header_bytes.get_bytes(dir_info_size);
        let dir_info = DirectoryInfo::from_bytes(&dir_info_bytes);

        all_dir_infos.push(dir_info);
    }

    let paths_and_sizes: Vec<(String, Option<u64>)> = all_dir_infos.iter()
        .flat_map(|dirinfo| dirinfo.get_paths_and_sizes())
        .collect();

    for (path, size) in paths_and_sizes
    {
        match size
        {
            None => create_dir(path).unwrap(),
            Some(size) => extract_file_content(path, size, &mut reader),
        }
    }

    Ok(())
}
