use crate::compress::lz77::{LONG_BUFFER_SIZE, SHORT_BUFFER_SIZE};
use crate::io_utils::universal_reader::UniversalReader;

pub struct CompressionWindow
{
    long_buffer: Vec<u8>,
    short_buffer: Vec<u8>,
    file_reader: UniversalReader,
}

impl CompressionWindow
{
    pub fn new(mut file_reader: UniversalReader) -> CompressionWindow
    {
        let mut short_buffer = Vec::new();
        for _ in 0..SHORT_BUFFER_SIZE
        {
            if let Some(byte) = file_reader.read_byte()
            {
                short_buffer.push(byte);
            }
            else
            {
                break;
            }
        }

        CompressionWindow
        {
            long_buffer: Vec::new(),
            short_buffer,
            file_reader,
        }
    }

    fn shift_once(&mut self)
    {
        if self.long_buffer.len() == LONG_BUFFER_SIZE
        {
            self.long_buffer.pop();
        }

        if !self.short_buffer.is_empty()
        {
            self.long_buffer.insert(0, self.short_buffer.remove(0));
        }

        if let Some(new_byte) = self.file_reader.read_byte()
        {
            self.short_buffer.push(new_byte);
        }
    }

    pub fn shift(&mut self, n: usize)
    {
        for _ in 0..n
        {
            self.shift_once();
        }
    }

    pub fn short_buffer_is_empty(&self) -> bool
    {
        self.short_buffer.is_empty()
    }

    fn data(&self) -> Vec<u8>
    {
        let long_buffer: Vec<u8> = self.long_buffer.iter().rev().cloned().collect();
        let short_buffer = self.short_buffer.clone();

        [long_buffer, short_buffer].concat()
    }

    pub fn find_longest_prefix(&self)
        ->
        (
            usize,  // offset between pattern start and last match (0 if no match)
            usize,          // prefix size (0 if no match)
            Option<u8>,     // next byte after the matching prefix
        )
    {
        let data = self.data();
        let data = data.as_slice();

        let long_len = self.long_buffer.len();
        let short_len = self.short_buffer.len();

        if long_len == 0
        {
            return (0, 0, data.get(0).cloned());
        }

        if short_len == 1
        {
            return (0, 0, data.get(long_len).cloned());
        }

        let short_start = long_len;

        // Starting from the longest possible prefix. Only proper prefix!
        for prefix_len in (1..short_len).rev()
        {
            let short_prefix = &data[short_start.. short_start + prefix_len];

            for start_index in (0..long_len).rev() // Looking for the last occurence.
            {
                let potential_match = &data[start_index..start_index + prefix_len];
                if potential_match == short_prefix
                {
                    let next_after = data.get(short_start + prefix_len).cloned();
                    let index = short_start - start_index;

                    return (index, prefix_len, next_after);
                }
            }
        }

        // If no pattern matches, return the start byte of the short buffer (maybe none).
        let next = data.get(short_start).cloned();
        (0, 0, next)
    }
}
