use num_bigint::BigUint;

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct BitAllocation {
    bits: Vec<bool>
}
impl BitAllocation {
    pub fn new(size: usize) -> Self {
        BitAllocation {
            bits: vec![false; size]
        }
    }
    pub fn new_zero() -> Self {
        Self::new(1)
    }
    pub fn get_length(&self) -> usize {
        self.bits.len()
    }
    pub fn get_bits(&self) -> &Vec<bool> {
        &self.bits
    }

    pub fn resize(&mut self, new_size: usize) {
        self.bits.resize(new_size, false);
    }
    pub fn to_big_num(&self) -> BigUint {
        let bytes: Vec<u8> = self.bits.chunks(8).map(|chunk| {
            let mut byte = 0u8;
            for (i, &bit) in chunk.iter().enumerate() {
                if bit { byte |= 1 << i; }
            }
            byte
        }).collect();
        BigUint::from_bytes_le(&*bytes)
    }
    pub fn apply_big_num(&mut self, num: &BigUint) {
        let bytes = num.to_bytes_le();
        self.bits.clear();

        for (byte_index, byte) in bytes.iter().enumerate() {
            for i in 0..8 {
                let bit_value = byte & (1 << i);
                let bool_bit_value = bit_value != 0;
                let target_index = byte_index * 8 + i;

                if target_index >= self.bits.len() {
                    return;
                } else {
                    self.bits[byte_index * 8 + i] = bool_bit_value;
                }
            }
        }
    }
    pub fn copy_from(&mut self, other: &BitAllocation) {
        for i in 0..self.get_length() {
            let other_bit_value = if i < other.get_length() {
                other.bits[i]
            } else {
                false
            };
            self.bits[i] = other_bit_value;
        }
    }
}