use std::fs;
use walkdir::WalkDir;
use serde::{Deserialize, Serialize};
use crate::io_utils::path_utils::get_superpath;

#[derive(Serialize, Deserialize)]
struct FilesystemEntryInfo
{
    path: String,
    size: Option<u64>,
}

impl FilesystemEntryInfo
{
    fn new(path: &str, superpath: &str) -> FilesystemEntryInfo
    {
        let size = if fs::metadata(path).unwrap().is_dir()
        {
            None
        }
        else
        {
            let file_size = fs::metadata(path).unwrap().len();
            Some(file_size)
        };

        let path = path.strip_prefix(superpath)
            .unwrap()
            .to_string();

        FilesystemEntryInfo
        {
            path,
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
        let directory_superpath = get_superpath(directory_path);

        let mut entry_infos = vec![];
        for entry in WalkDir::new(directory_path)
        {
            let path = entry.unwrap().path().to_str().unwrap().to_string()
                .replace("\\", "/");

            let entry_info = FilesystemEntryInfo::new(&path, &directory_superpath);
            entry_infos.push(entry_info);
        }

        DirectoryInfo
        {
            infos: entry_infos,
        }
    }

    pub fn to_bytes(&self) -> Result<Vec<u8>, ()>
    {
        let content = serde_json::to_string(self)
            .map_err(|_| ())?
            .into_bytes();

        let bytes_size = (content.len() as u64).to_be_bytes().to_vec();

        let bytes = [bytes_size, content].concat();
        Ok(bytes)
    }

    pub fn from_bytes(bytes: &[u8]) -> DirectoryInfo
    {
        let data_string = String::from_utf8(Vec::from(bytes))
            .unwrap();

        serde_json::from_str(&data_string)
            .unwrap()
    }

    pub fn get_paths_and_sizes(&self) -> Vec<(String, Option<u64>)>
    {
        let mut result = vec![];

        for info in &self.infos
        {
            let path = info.path.clone();
            let size = info.size;

            result.push((path, size));
        }

        result
    }
}
