use std::fs;
use walkdir::WalkDir;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct FilesystemEntryInfo
{
    path: String,
    size: Option<u64>,
}

impl FilesystemEntryInfo
{
    fn new(path: &str) -> FilesystemEntryInfo
    {
        let size = if fs::metadata(&path).unwrap().is_dir()
        {
            None
        }
        else
        {
            let file_size = fs::metadata(&path).unwrap().len();
            Some(file_size)
        };

        FilesystemEntryInfo
        {
            path: path.to_string(),
            size,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct DirectoryInfo
{
    infos: Vec<FilesystemEntryInfo>,
}

impl DirectoryInfo
{
    pub fn new(directory_path: &str) -> DirectoryInfo
    {
        let mut entry_infos = vec![];
        for entry in WalkDir::new(directory_path)
        {
            let path = entry.unwrap().path().to_str().unwrap().to_string();
            let entry_info = FilesystemEntryInfo::new(&path);
            entry_infos.push(entry_info);
        }

        DirectoryInfo
        {
            infos: entry_infos,
        }
    }

    pub fn to_bytes(&self) -> Vec<u8>
    {
        let bytes = serde_json::to_string(self).unwrap()
            .into_bytes();

        let bytes_size = (bytes.len() as u64).to_be_bytes().to_vec();

        [bytes_size, bytes].concat()
    }

    pub fn get_all_file_paths(&self) -> Vec<String>
    {
        self.infos.iter().map(|line| line.path.clone()).collect()
    }
}
