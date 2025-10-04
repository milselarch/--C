extern crate num_bigint;
extern crate num_traits;

use num_bigint::BigInt;

#[derive(Clone, Debug)]
pub enum ALUOperations {
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulo,
}

#[derive(Clone, Debug)]
pub enum PotatoCodes {
    MovOutputToStack(usize),
    MovStackToOutput(usize),
    MovStackToInputA(usize),
    MovStackToInputB(usize),
    Operate(ALUOperations),
    ResizeOutput(usize),
}

pub struct PotatoCPU {
    pub stack: Vec<u32>,
    pub input_register_a: BigInt,
    pub input_register_b: BigInt,
    pub scratch_registers: [BigInt; 2],
    pub output_register: BigInt,
}

