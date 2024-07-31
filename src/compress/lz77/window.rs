use crate::io_utils::universal_reader::UniversalReader;

const LONG_BUFFER_SIZE: usize = 1 << 15;
const SHORT_BUFFER_SIZE: usize = 258;

pub struct Window
{
    long_buffer: Vec<u8>,
    short_buffer: Vec<u8>,
    file_reader: UniversalReader,
}

impl Window
{
    pub fn new(mut file_reader: UniversalReader) -> Window
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

        Window
        {
            long_buffer: Vec::new(),
            short_buffer,
            file_reader,
        }
    }

    fn shift(&mut self)
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

    pub fn shift_n_times(&mut self, n: usize)
    {
        for _ in 0..n
        {
            self.shift();
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
            Option<u8>,     // next byte after
        )
    {
        let data = self.data();
        let data = data.as_slice();

        let text_range = 0..self.long_buffer.len();
        let pattern_range =
            self.long_buffer.len()..self.long_buffer.len() + self.short_buffer.len();

        if text_range.is_empty()
        {
            return (0, 0, data.get(pattern_range.start).cloned());
        }

        if pattern_range.len() == 1
        {
            return (0, 0, data.get(pattern_range.start).cloned());
        }

        let pattern_len = pattern_range.end - pattern_range.start;

        for prefix_len in (1..=pattern_len - 1).rev() // Starting from the longest possible prefix.
        {
            let prefix = &data[pattern_range.start..pattern_range.start + prefix_len];

            for start_index in (text_range.start..text_range.end).rev() // Looking for the last occurence.
            {
                let potential_match = &data[start_index..start_index + prefix_len];
                if potential_match == prefix
                {
                    let next_after = data.get(pattern_range.start + prefix_len)
                        .cloned();
                    let index = pattern_range.start - start_index;

                    return (index, prefix_len, next_after);
                }
            }
        }

        let next = data.get(pattern_range.start).cloned();
        (0, 0, next)
    }
}
