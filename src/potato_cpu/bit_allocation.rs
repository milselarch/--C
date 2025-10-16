use std::ops::{Add, Shl, Shr};
use arbitrary_int::u4;
use num_bigint::BigUint;
use num_traits::ToPrimitive;

pub trait BitAllocation {
    fn get_length(&self) -> usize;
    fn get_bits(&self) -> &Vec<bool>;
    fn to_big_num(&self) -> BigUint;
    fn apply_big_num(
        &mut self, num: &BigUint
    );
    fn copy_from(&mut self, other: &Self);
    fn set(&mut self, index: usize, value: bool);
    fn get(&self, index: usize) -> bool {
        self.get_bits()[index]
    }
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
    pub fn new_from(bits: Vec<bool>) -> Self {
        FixedBitAllocation {
            bit_allocation: GrowableBitAllocation { bits }
        }
    }
    pub fn new_zero(size: usize) -> Self {
        Self::new(size)
    }
    pub fn new_one(size: usize) -> Self {
        let mut allocation = Self::new(size);
        allocation.bit_allocation.bits[0] = true;
        allocation
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
    fn apply_big_num(&mut self, num: &BigUint) {
        self.bit_allocation.apply_big_num(num);
    }
    fn copy_from(&mut self, other: &FixedBitAllocation) {
        self.bit_allocation.copy_from(&other.bit_allocation);
    }
    fn set(&mut self, index: usize, value: bool) {
        self.bit_allocation.set(index, value);
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct GrowableBitAllocation {
    bits: Vec<bool>
}
impl GrowableBitAllocation {
    pub fn new(size: usize) -> Self {
        Self::new_from(vec![false; size])
    }
    pub fn new_from(bits: Vec<bool>) -> Self {
        GrowableBitAllocation { bits }
    }
    pub fn new_zero() -> Self {
        Self::new(1)
    }
    pub fn from_fixed_allocations(allocations: &Vec<FixedBitAllocation>) -> Self {
        // concatenate fixed allocations into one growable allocation
        let mut result = Self::new(0);
        for allocation in allocations.iter() {
            result.append(allocation);
        }
        result
    }
    pub(crate) fn from_big_num(num: &BigUint) -> Self {
        let mut allocation = Self::new(0);
        allocation.apply_big_num(num);
        allocation
    }
    pub fn from_num(num: usize) -> Self {
        let big_num = BigUint::from(num);
        GrowableBitAllocation::from_big_num(&big_num)
    }
    pub fn from_i64(num: i64) -> Self {
        if num >= 0 {
            GrowableBitAllocation::from_num(num as usize)
        } else {
            let mut allocation = GrowableBitAllocation::from_num((-num) as usize);
            allocation.apply_twos_complement();
            allocation
        }
    }
    pub fn to_i64(&self) -> Option<i64> {
        let big_num = self.to_big_num();
        big_num.to_i64()
    }
    pub fn new_from_bool(value: bool) -> Self {
        GrowableBitAllocation::new_from(vec![value])
    }
    pub fn new_from_num(num: usize) -> Self {
        let big_num = BigUint::from(num);
        GrowableBitAllocation::from_big_num(&big_num)
    }
    pub fn get_length(&self) -> usize {
        self.bits.len()
    }
    pub fn apply_twos_complement(&mut self) -> &mut Self {
        // flip all bits
        for bit in self.bits.iter_mut() {
            *bit = !*bit;
        }
        self.increment()
    }
    pub fn increment(&mut self) -> &mut Self {
        /*
        TODO: I feel like implementing addition directly would be relatively
            straightforward and more efficient than converting to BigUint
            (and needing to import num-bigint just for that)
        */
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
        if carry {
            self.bits.push(true);
        }
        self
    }

    pub const fn translate_bool_op(&self, a: bool, b: bool, bool_operation: u4) -> bool {
        match bool_operation.value() {
            0 => false,             // 0000
            1 => a & b,             // 0001
            2 => a & !b,            // 0010
            3 => a,                 // 0011
            4 => !a & b,            // 0100
            5 => b,                 // 0101
            6 => a ^ b,             // 0110
            7 => a | b,             // 0111
            8 => !(a | b),          // 1000
            9 => !(a ^ b),          // 1001
            10 => !b,               // 1010
            11 => a | !b,           // 1011
            12 => !a,               // 1100
            13 => !a | b,           // 1101
            14 => !(a & b),         // 1110
            15 => true,             // 1111
            _ => unreachable!(),    // u4 can only be in [0, 15]
        }
    }

    pub fn apply_boolean_operation(
        &self, other: &GrowableBitAllocation, op: u4
    ) -> Self {
        let max_length = usize::max(self.get_length(), other.get_length());
        let mut a = self.clone();
        let mut b = other.clone();
        a.resize(max_length);
        b.resize(max_length);

        let mut result_bits = Vec::with_capacity(max_length);
        for i in 0..max_length {
            let a_bit = a.bits[i];
            let b_bit = b.bits[i];
            let result_bit = self.translate_bool_op(a_bit, b_bit, op);
            result_bits.push(result_bit);
        }

        GrowableBitAllocation::new_from(result_bits)
    }
    pub fn resize(&mut self, new_size: usize) -> &mut Self {
        self.bits.resize(new_size, false);
        self
    }
    pub fn resize_modulo(&mut self, size_modulo: usize) -> &mut Self {
        let current_size = self.bits.len();
        let modulo_size = current_size % size_modulo;

        if modulo_size != 0 {
            let new_size = current_size + (size_modulo - modulo_size);
            assert!(new_size > current_size);
            self.resize(new_size);
        }
        self
    }
    pub fn signed_resize(&mut self, new_size: usize) -> &mut Self {
        let sign_bit = *self.bits.last().unwrap();
        self.bits.resize(new_size, sign_bit);
        self
    }
    pub fn auto_shrink(&mut self) -> &mut Self {
        // remove trailing zeros
        while self.bits.len() > 1 && !self.bits.last().unwrap() {
            self.bits.pop();
        }
        self
    }
    pub fn to_fixed_allocation(&self) -> FixedBitAllocation {
        FixedBitAllocation {
            bit_allocation: self.clone()
        }
    }
    pub fn split(&self, split_size: usize) -> Vec<FixedBitAllocation> {
        /*
        Chops up the allocation into smaller fixed-size allocations
        of length split_size. If the last chunk is smaller than split_size,
        it is padded with the most significant bit (sign bit) (little-endian)
        to reach the desired size.
        */
        let mut result = Vec::new();
        let mut index = 0;

        while index < self.bits.len() {
            let end_index = usize::min(index + split_size, self.bits.len());
            let mut chunk_bits = self.bits[index..end_index].to_vec();
            let msb = *chunk_bits.last().unwrap();
            chunk_bits.resize(split_size, msb);

            let chunk = FixedBitAllocation::new_from(chunk_bits);
            result.push(chunk);
            index += split_size;
        }
        result
    }
    pub fn append(&mut self, other: &FixedBitAllocation) {
        self.bits.extend_from_slice(other.get_bits());
    }
    pub fn reverse(&mut self) -> &mut Self {
        self.bits.reverse();
        self
    }
}
impl BitAllocation for GrowableBitAllocation {
    fn get_length(&self) -> usize {
        self.bits.len()
    }
    fn get_bits(&self) -> &Vec<bool> {
        &self.bits
    }
    fn to_big_num(&self) -> BigUint {
        let bytes: Vec<u8> = self.bits.chunks(8).map(|chunk| {
            let mut byte = 0u8;
            for (i, &bit) in chunk.iter().enumerate() {
                if bit { byte |= 1 << i; }
            }
            byte
        }).collect();
        BigUint::from_bytes_le(&*bytes)
    }
    fn apply_big_num(&mut self, num: &BigUint) {
        let bytes = num.to_bytes_le();
        self.resize(bytes.len() * 8);
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

        self.auto_shrink();
    }
    fn copy_from(&mut self, other: &GrowableBitAllocation) {
        for i in 0..self.get_length() {
            let other_bit_value = if i < other.get_length() {
                other.bits[i]
            } else {
                false
            };
            self.bits[i] = other_bit_value;
        }
    }
    fn set(&mut self, index: usize, value: bool) {
        self.bits[index] = value;
    }
}
impl Add for GrowableBitAllocation {
    type Output = GrowableBitAllocation;

    fn add(self, other: GrowableBitAllocation) -> GrowableBitAllocation {
        // TODO: in retrospect was bignum actually necessary
        let sum = self.to_big_num() + other.to_big_num();
        GrowableBitAllocation::from_big_num(&sum)
    }
}
impl Add for &GrowableBitAllocation {
    type Output = GrowableBitAllocation;

    fn add(self, other: &GrowableBitAllocation) -> GrowableBitAllocation {
        let sum = self.to_big_num() + other.to_big_num();
        GrowableBitAllocation::from_big_num(&sum)
    }
}
impl Shl for &GrowableBitAllocation {
    type Output = GrowableBitAllocation;

    fn shl(self, shift: &GrowableBitAllocation) -> GrowableBitAllocation {
        let shift_amount_opt = shift.to_big_num().to_usize();
        let shift_amount = match shift_amount_opt {
            Some(val) => val,
            None => return GrowableBitAllocation::new_zero(),
        };
        let bits = self.bits.clone();
        let result_bits = &bits[shift_amount..];
        GrowableBitAllocation::new_from(result_bits.to_vec())
    }
}
impl Shr for &GrowableBitAllocation {
    type Output = GrowableBitAllocation;

    fn shr(self, shift: &GrowableBitAllocation) -> GrowableBitAllocation {
        let shift_amount_opt = shift.to_big_num().to_usize();
        let shift_amount = match shift_amount_opt {
            Some(val) => val,
            None => return GrowableBitAllocation::new_zero(),
        };
        let mut result_bits = vec![false; shift_amount];
        result_bits.extend_from_slice(&self.bits);
        GrowableBitAllocation::new_from(result_bits)
    }
}