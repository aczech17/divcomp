use std::fs;
use std::fs::File;
use walkdir::WalkDir;
use crate::archive::archive_header::ArchiveHeader;
use crate::archive::directory_info::DirectoryInfo;
use crate::io_utils::byte_writer::ByteWriter;
use crate::io_utils::universal_reader::UniversalReader;

fn save_file_to_archive(file_path: &str, output: &mut ByteWriter) -> Result<(), String>
{
    let input_file = File::open(file_path)
        .map_err(|_| format!("Could not open file {}", file_path))?;

    let mut reader = UniversalReader::new(input_file);

    while let Some(byte) = reader.read_byte()
    {
        output.write_byte(byte);
    }

    Ok(())
}

pub fn pack(input_paths: Vec<String>, output_file: File) -> Result<(), String>
{
    let all_directory_infos: Vec<DirectoryInfo> = input_paths.iter()
        .map(|path| DirectoryInfo::new(path))
        .collect();

    let archive_header = ArchiveHeader::new(all_directory_infos)
        .map_err(|_| "Could not create archive header.")?;

    let mut output_writer = ByteWriter::new(output_file)?;

    for byte in archive_header.to_bytes()
    {
        output_writer.write_byte(byte);
    }

    // Save the files to the archive. Now the full paths are needed.
    for input_path in input_paths
    {
        for path in WalkDir::new(input_path)
        {
            let path = path.unwrap().path().to_str().unwrap().to_string();
            if !fs::metadata(&path).unwrap().is_dir()
            {
                save_file_to_archive(&path, &mut output_writer)?;
            }
        }
    }

    Ok(())
}
