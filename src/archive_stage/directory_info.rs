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
    iterator_index: usize,
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
            iterator_index: 0,
        }
    }

    pub fn to_bytes(&self) -> Vec<u8>
    {
        let bytes = serde_json::to_string(self).unwrap()
            .into_bytes();

        let bytes_size = (bytes.len() as u64).to_be_bytes().to_vec();

        [bytes_size, bytes].concat()
    }

    pub fn from_bytes(bytes: &[u8]) -> DirectoryInfo
    {
        let data_string = String::from_utf8(Vec::from(bytes))
            .unwrap();

        serde_json::from_str(&data_string)
            .unwrap()
    }

    pub fn get_all_file_paths(&self) -> Vec<String>
    {
        self.infos.iter().map(|line| line.path.clone()).collect()
    }

    pub fn rewind(&mut self)
    {
        self.iterator_index = 0;
    }
}

impl Iterator for DirectoryInfo
{
    type Item = (String, Option<u64>);

    fn next(&mut self) -> Option<Self::Item>
    {
        if self.iterator_index >= self.infos.len()
        {
            return None;
        }

        let entry = &self.infos[self.iterator_index];
        self.iterator_index += 1;
        Some((entry.path.clone(), entry.size))
    }
}
