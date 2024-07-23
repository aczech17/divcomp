use crate::archive::directory_info::DirectoryInfo;

pub struct ArchiveHeader
{
    header_size: u64,
    header_data: Vec<u8>,
}

impl ArchiveHeader
{
    pub fn new(directory_infos: Vec<DirectoryInfo>) -> Result<ArchiveHeader, ()>
    {
        let mut data = vec![];
        for info in directory_infos
        {
            let mut info_bytes = info.to_bytes()?;
            data.append(&mut info_bytes);
        }

        let data_size = data.len() as u64;

        let archive_header = ArchiveHeader
        {
            header_size: data_size,
            header_data: data,
        };

        Ok(archive_header)
    }

    pub fn to_bytes(&self) -> Vec<u8>
    {
        [self.header_size.to_be_bytes().to_vec(), self.header_data.clone()].concat()
    }
}
