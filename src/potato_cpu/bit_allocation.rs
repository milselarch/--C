use num_bigint::BigUint;

pub trait BitAllocation {
    fn get_length(&self) -> usize;
    fn get_bits(&self) -> &Vec<bool>;
    fn to_big_num(&self) -> BigUint;
    fn apply_big_num(
        &mut self, num: &BigUint
    );
    fn copy_from(&mut self, other: &Self);
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct FixedBitAllocation {
    bit_allocation: GrowableBitAllocation
}
impl FixedBitAllocation {
    pub fn new(size: usize) -> Self {
        FixedBitAllocation {
            bit_allocation: GrowableBitAllocation::new(size)
        }
    }
    pub fn new_zero(size: usize) -> Self {
        Self::new(size)
    }
    fn to_growable(&self) -> GrowableBitAllocation {
        self.bit_allocation.clone()
    }
}
impl BitAllocation for FixedBitAllocation {
    fn get_length(&self) -> usize {
        self.bit_allocation.get_length()
    }
    fn get_bits(&self) -> &Vec<bool> {
        self.bit_allocation.get_bits()
    }
    fn to_big_num(&self) -> BigUint {
        self.bit_allocation.to_big_num()
    }
    fn apply_big_num(
        &mut self, num: &BigUint
    ) {
        self.bit_allocation.apply_big_num(num, false);
    }
    fn copy_from(&mut self, other: &FixedBitAllocation) {
        self.bit_allocation.copy_from(&other.bit_allocation);
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct GrowableBitAllocation {
    bits: Vec<bool>
}
impl GrowableBitAllocation {
    pub fn new(size: usize) -> Self {
        GrowableBitAllocation {
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
    pub fn apply_twos_complement(&mut self) {
        // flip all bits
        for bit in self.bits.iter_mut() {
            *bit = !*bit;
        }
        // add one
        let mut carry = true;
        for bit in self.bits.iter_mut() {
            if carry {
                if *bit {
                    *bit = false;
                } else {
                    *bit = true;
                    carry = false;
                }
            } else {
                break;
            }
        }
    }

    pub fn resize(&mut self, new_size: usize) {
        self.bits.resize(new_size, false);
    }
    pub fn signed_resize(&mut self, new_size: usize) {
        let sign_bit = *self.bits.last().unwrap();
        self.bits.resize(new_size, sign_bit);
    }
    pub fn auto_shrink(&mut self) {
        // remove trailing zeros
        while self.bits.len() > 1 && !self.bits.last().unwrap() {
            self.bits.pop();
        }
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
    pub fn apply_big_num(
        &mut self, num: &BigUint, auto_resize: bool
    ) {
        let bytes = num.to_bytes_le();
        if auto_resize {
            self.resize(bytes.len() * 8);
        }
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

        if auto_resize {
            self.auto_shrink();
        }
    }
    pub fn copy_from(&mut self, other: &GrowableBitAllocation) {
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