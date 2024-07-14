use serde::Serialize;
use crate::archive_stage::directory_info::DirectoryInfo;
use crate::io_utils::byte_writer::ByteWriter;

pub fn archive(input_paths: Vec<String>, output_filename: String)
{
    let mut output_writer = ByteWriter::new(&output_filename)
        .unwrap();
    let mut all_paths = vec![];

    for input_path in input_paths
    {
        let directory_info = DirectoryInfo::new(&input_path);
        let mut directory_paths = directory_info.get_all_file_paths();

        let directory_info = serde_json::to_string(&directory_info)
            .unwrap();
        directory_info.into_bytes().iter().for_each(|&byte| output_writer.write_byte(byte));

    }
}
