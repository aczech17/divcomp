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

        let pattern_start = long_len;
        let pattern_len = short_len;

        // Starting from the longest possible prefix. Only proper prefix!
        for prefix_len in (1..pattern_len).rev()
        {
            let pattern_prefix = &data[pattern_start .. pattern_start + prefix_len];

            for start_index in (0..long_len).rev() // Looking for the last occurence.
            {
                let potential_match = &data[start_index..start_index + prefix_len];
                if potential_match == pattern_prefix
                {
                    let next_after = data.get(pattern_start + prefix_len).cloned();
                    let index = pattern_start - start_index;

                    return (index, prefix_len, next_after);
                }
            }
        }

        // If not pattern matches, return the next byte after the pattern (maybe none).
        let next = data.get(pattern_start).cloned();
        (0, 0, next)
    }
}
