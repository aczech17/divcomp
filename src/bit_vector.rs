pub type Bit = u8;

pub struct BitVector
{
    data: Vec<u8>,
    bits_count: usize,
}

impl BitVector
{
    pub fn new() -> Self
    {
        Self
        {
            data: vec![],
            bits_count: 0,
        }
    }

    pub fn push_bit(&mut self, bit: Bit)
    {
        if self.bits_count % 8 == 0
        {
            self.data.push(0);
        }

        let byte_number = self.bits_count / 8;
        let bit_number = self.bits_count % 8;

        self.data[byte_number] |= bit << (7 - bit_number);
        self.bits_count += 1;
    }

    pub fn push_byte(&mut self, byte: u8)
    {
        for bit_number in 0..8
        {
            let bit = (byte >> (7 - bit_number)) & 1;
            self.push_bit(bit);
        }
    }

    pub fn pop(&mut self)
    {
        let byte_number = (self.bits_count - 1) / 8;
        let bit_number = (self.bits_count - 1) % 8;

        let clearing_mask = !(1 << (7 - bit_number));
        self.data[byte_number] &= clearing_mask;

        self.bits_count -= 1;

        if self.bits_count % 8 == 0
        {
            self.data.pop();
        }
    }

    pub fn get(&self, index: usize) -> Bit
    {
        let byte_number = index / 8;
        let bit_number = index % 8;

        (self.data[byte_number] >> (7 - bit_number)) & 1
    }
}
