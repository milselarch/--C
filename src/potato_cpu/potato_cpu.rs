extern crate num_bigint;
extern crate num_traits;

use crate::potato_cpu::bit_allocation::{
    BitAllocation, FixedBitAllocation, GrowableBitAllocation
};
use arbitrary_int::{u4, UInt};
use num_bigint::BigInt;
use std::cmp::PartialEq;
use std::collections::HashMap;
use std::ops::Add;

const AND_OP: UInt<u8, 4> = u4::new(0b1000);
const OR_OP: UInt<u8, 4> = u4::new(0b1110);
/*
If an operation has a worse time complexity when implemented
using assembly instead of directly implementing it in the cellular automaton,
then it should be supported natively as an ALU operation.
*/
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ALUOperations {
    // O(n), assembly is O(n^2) cause n reapplications of carry
    Add,
    ReverseBits,
    /*
    Perhaps the instruction itself could just encode a mapping
    of all (bit_a, bit_b) -> result_bit for all input combinations
    within the CA
    */
    BitwiseNOperation(u4),
    // O(n*log(n)), assembly implementation would be O(n^2)
    ShiftLeft,
    ShiftCircularLeft,
    CompareGreaterThan,
    /*
    Return the length of the input register data in bits
    O(n) is CA, assembly is O(n^2) (shift left until zero)
    Also doubles as a way to get log2(input) for input > 0
    */
    GetLength,
    // shrink / grow A to size B
    Resize,
    // grow A to be a multiple of size B
    ResizeModulo
    /*
    - twos complement is just flipping all bits and adding 1
      so O(n) + O(n) = O(n)
    - subtract is just a + b's twos complement
      also O(n) + O(n) = O(n)
    - I think circular right is not needed since
      circular left by k is the same as circular right by (n - k)
    - use assembly to implement times, divide, modulo lol
      O(n^2) in both native CA and assembly
    - write-through is just input | 0
    - ~input is just input NAND input
    - truthiness is just checking if input > 0
    */
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub enum Registers {
    InputA,
    InputB,
    // same purpose as EDI / ESI ... registers in System V ABI
    FunctionInput,
    StackPointer,
    BasePointer,
    Scratch(u8),
    Output,
    FunctionReturn
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MovStackToRegister {
    stack_address: usize,
    num_stack_addresses: usize,
    register: Registers
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StrideMovRegisterToStack {
    register: Registers,
    start_stack_address: usize,
    stride: usize
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StrideMovStackToRegister {
    start_stack_address: usize,
    stride: usize,
    num_stack_addresses: usize,
    register: Registers
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PotatoCodes {
    // register, stack address
    MovRegisterToStack(Registers, usize),
    // stack address, num stack addresses to copy, register
    MovStackToRegister(MovStackToRegister),
    CopyRegisterToRegister(Registers, Registers),

    StrideMovRegisterToStack(StrideMovRegisterToStack),
    StrideMovStackToRegister(StrideMovStackToRegister),

    Operate(ALUOperations),
    DataValue(GrowableBitAllocation),
    // move instruction data value to register
    MovDataValueToRegister(usize, Registers),
    // resize value in output register to fit in stack

    ResizeOutput(usize)
}

#[derive(Clone, Debug)]
pub struct StepResult {
    pub halted: bool,
    pub time_steps: usize
}

#[derive(Clone, Debug)]
pub struct PotatoSpec {
    instructions: Vec<PotatoCodes>,
    num_scratch_registers: u8,
    stack_width: u16,
}

/*
Stack width is finite but registers are infinite size
*/
pub struct PotatoCPU {
    pub spec: PotatoSpec,
    pub stack: Vec<FixedBitAllocation>,
    pub time_steps: usize,
    pub program_counter: usize,
    pub registers: HashMap<Registers, GrowableBitAllocation>,
    pub halted: bool
}

impl PotatoCPU {
    pub fn new(spec: PotatoSpec) -> PotatoCPU {
        PotatoCPU {
            stack: vec![],
            spec,
            time_steps: 0,
            program_counter: 0,
            registers: HashMap::new(),
            halted: false
        }
    }

    pub fn get_instructions(&self) -> &Vec<PotatoCodes> {
        &self.spec.instructions
    }
    pub fn get_num_stack_registers(&self) -> u8 {
        self.spec.num_scratch_registers
    }
    pub fn spawn_new_stack_value(&self) -> FixedBitAllocation {
        FixedBitAllocation::new(self.spec.stack_width as usize)
    }

    pub fn assign_to_stack(&mut self, index: usize, value: FixedBitAllocation) {
        if index >= self.stack.len() {
            let blank_stack_value = self.spawn_new_stack_value();
            self.stack.resize(index + 1, blank_stack_value);
        }
        self.stack[index].copy_from(&value);
    }
    pub fn read_from_stack(&self, index: usize) -> FixedBitAllocation {
        if index < self.stack.len() {
            self.stack[index].clone()
        } else {
            self.spawn_new_stack_value()
        }
    }
    fn validate_register(&self, reg: &Registers) {
        if let Registers::Scratch(scratch_register_no) = &reg {
            if *scratch_register_no >= self.spec.num_scratch_registers {
                panic!(
                    "Scratch register number {} out of bounds (max {})",
                   scratch_register_no, self.spec.num_scratch_registers - 1
                );
            }
        }
    }
    pub fn load_register(&mut self, reg: Registers) -> &GrowableBitAllocation {
        self.validate_register(&reg);
        self.registers.entry(reg).or_insert(
            GrowableBitAllocation::new(0)
        )
    }
    pub fn load_register_mut(&mut self, reg: Registers) -> &mut GrowableBitAllocation {
        self.validate_register(&reg);
        self.registers.entry(reg).or_insert(
            GrowableBitAllocation::new(0)
        )
    }

    pub fn step(&mut self) -> StepResult {
        if self.halted {
            return StepResult {
                halted: true,
                time_steps: self.time_steps,
            };
        }

        self.time_steps += 1;
        self.program_counter += 1;
        let instructions = self.get_instructions();
        if self.program_counter >= instructions.len() {
            self.halted = true;
        }

        let instruction = instructions[self.program_counter];
        match instruction {
            PotatoCodes::MovRegisterToStack(reg, index) => {
                let register_value = self.load_register(reg);
                let chunks = register_value.split(self.spec.stack_width as usize);
                for (i, chunk) in chunks.into_iter().enumerate() {
                    self.assign_to_stack(index + i, chunk);
                }
            },
            PotatoCodes::MovStackToRegister(params) => {
                let register = self.load_register_mut(params.register);
                for i in 0..params.num_stack_addresses {
                    let stack_value = self.read_from_stack(params.stack_address + i);
                    register.append(&stack_value);
                }
            },
            PotatoCodes::Operate(op) => {
                let a = self.load_register(Registers::InputA);
                let b = self.load_register(Registers::InputB);
                let a_size = a.get_length();
                let b_size = b.get_length();
                let max_size = std::cmp::max(a_size, b_size);

                let result = match op {
                    ALUOperations::Add => a + b,
                    ALUOperations::ReverseBits => {
                        *a.clone().reverse()
                    },
                    ALUOperations::BitwiseNOperation(op_code) => {
                        todo!()
                    },
                };
                self.registers.insert(Registers::Output, result);
            },
            PotatoCodes::ResizeOutput(size) => {
                let output = self.registers.get(&Registers::Output).cloned().unwrap_or(BigInt::from(0));
                let resized = output & ((BigInt::from(1) << (size * 8)) - 1);
                self.registers.insert(Registers::Output, resized);
            }
            PotatoCodes::DataValue(..) => {
                // no-op
            }
            PotatoCodes::MovDataValueToRegister(index, reg) => {
                let instruction = &self.instructions[*index];
                if let PotatoCodes::DataValue(value) = instruction {
                    self.registers.insert(reg.clone(), value.clone());
                } else {
                    panic!("Expected DataValue at index {}", index)
                }
            }
        }

        StepResult {
            halted: self.halted,
            time_steps: self.time_steps,
            return_value: self.registers.get(&Registers::FunctionReturn).cloned()
        }
    }

    pub fn add_inputs(
        input_a: &GrowableBitAllocation, input_b: &GrowableBitAllocation
    ) -> GrowableBitAllocation {
        let a = input_a.to_big_num();
        let b = input_b.to_big_num();
        let result = a + b;
        GrowableBitAllocation::from_big_num(&result)
    }
    pub fn subtract_inputs(
        input_a: &GrowableBitAllocation, input_b: &GrowableBitAllocation
    ) -> GrowableBitAllocation {
        let a = input_a.to_big_num();
        let b = input_b.to_big_num();
        let result = a - b;
        GrowableBitAllocation::from_big_num(&result)
    }
}