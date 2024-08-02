use crate::compress::lz77::LONG_BUFFER_SIZE;
use crate::io_utils::byte_writer::ByteWriter;
use crate::io_utils::universal_reader::UniversalReader;

const WHOLE_BUFFER_SIZE: usize = 2 * LONG_BUFFER_SIZE;

struct DecompressionBuffer
{
    data: Vec<u8>,
    input: UniversalReader,
    output: ByteWriter,
}

impl DecompressionBuffer
{
    pub fn new(input: UniversalReader, output: ByteWriter) -> DecompressionBuffer
    {
        DecompressionBuffer
        {
            data: Vec::with_capacity(WHOLE_BUFFER_SIZE),
            input,
            output,
        }
    }

    /// Gets buffer from range [from, to), writes them to the file and erases them.
    fn flush(&mut self, from: usize, to: usize)
    {
        for index in from..to
        {
            self.output.write_byte(self.data[index]);
        }

        self.data.drain(from..to);
    }

    pub fn push(&mut self, value: u8)
    {
        if self.data.len() == WHOLE_BUFFER_SIZE
        {
            self.flush(0, LONG_BUFFER_SIZE);
        }

        self.data.push(value);
    }

    fn load_bytes(&mut self)
    {
        // read offset
        let b1 = self.input.read_byte().unwrap();
        let b2 = self.input.read_byte().unwrap();
        let offset = (((b1 as u16) << 8) | (b2 as u16)) as usize;

        // read bytes count
        let b1 = self.input.read_byte().unwrap();
        let b2 = self.input.read_byte().unwrap();
        let count = (((b1 as u16) << 8) | (b2 as u16)) as usize;

        let byte_after = self.input.read_byte();

        if offset > 0
        {
            let repeats_count = count / offset;
            let remainder = count % offset;

            let data_size = self.data.len();
            let mut added_bytes = Vec::new();
            for _ in 0..repeats_count
            {
                added_bytes.extend_from_slice(&self.data[data_size - offset..]);
            }
            added_bytes.extend_from_slice
                (&self.data[data_size - offset .. data_size - offset + remainder]);

            for byte in added_bytes
            {
                self.push(byte);
            }
        }

        if let Some(byte) = byte_after
        {
            self.push(byte);
        }
    }

    pub fn get(&self, index: usize) -> u8
    {
        if index >= LONG_BUFFER_SIZE
        {
            panic!("Index exceeds the dictionary buffer size.");
        }

        self.data[index]
    }
}

impl Drop for DecompressionBuffer
{
    fn drop(&mut self)
    {
        self.flush(0, self.data.len());
    }
}
