use std::fs;
use std::fs::File;
use crate::archive_stage::directory_info::DirectoryInfo;
use crate::io_utils::byte_writer::ByteWriter;
use crate::io_utils::universal_reader::UniversalReader;

fn save_file_to_archive(file_path: &str, output: &mut ByteWriter)
{
    let input_file = File::open(file_path)
        .unwrap();

    let mut reader = UniversalReader::new(input_file);

    while let Some(byte) = reader.read_byte()
    {
        output.write_byte(byte);
    }
}

pub fn archive(input_paths: Vec<String>, output_filename: String)
{
    let mut output_writer = ByteWriter::new(&output_filename)
        .unwrap();
    let mut all_paths: Vec<String> = vec![];

    for input_path in input_paths
    {
        let directory_info = DirectoryInfo::new(&input_path);

        // Save all paths from this directory for future use.
        directory_info.get_all_file_paths().iter()
            .for_each(|path| all_paths.push(path.to_string()));

        // Write directory info to the output file.
        serde_json::to_string(&directory_info)
            .unwrap()
            .into_bytes().iter()
            .for_each(|&byte| output_writer.write_byte(byte));

        // 0 as a separator between directory infos.
        output_writer.write_byte(0);
    }
    output_writer.write_byte(1); // 1 is the end of metadata.

    for file_path in all_paths
    {
        if !fs::metadata(&file_path).unwrap().is_dir()
        {
            save_file_to_archive(&file_path, &mut output_writer);
        }
    }
}
