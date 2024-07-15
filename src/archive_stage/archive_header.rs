use crate::archive_stage::directory_info::DirectoryInfo;

pub struct ArchiveHeader
{
    header_size: u64,
    header_data: Vec<u8>,
}

impl ArchiveHeader
{
    pub fn new(directory_infos: Vec<DirectoryInfo>) -> ArchiveHeader
    {
        let data: Vec<u8> = directory_infos.iter()
            .flat_map(|info| serde_json::to_string(info)
                .unwrap().into_bytes())
            .collect();

        let data_size = data.len() as u64;

        ArchiveHeader
        {
            header_size: data_size,
            header_data: data,
        }
    }

    pub fn to_bytes(&self) -> Vec<u8>
    {
        [self.header_size.to_be_bytes().to_vec(), self.header_data.clone()].concat()
    }
}
