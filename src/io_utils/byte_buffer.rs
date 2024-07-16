pub struct ByteBuffer
{
    data: Vec<u8>,
    index: usize,
}

impl ByteBuffer
{
    pub fn new(data: Vec<u8>) -> ByteBuffer
    {
        ByteBuffer
        {
            data,
            index: 0,
        }
    }

    pub fn end(&self) -> bool
    {
        self.index >= self.data.len()
    }

    pub fn get_bytes(&mut self, count: usize) -> Vec<u8>
    {
        let bytes: Vec<u8> = (0..count)
            .map(|i| self.data[self.index + i])
            .collect();

        self.index += count;
        bytes
    }
}
