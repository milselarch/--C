extern crate num_bigint;
extern crate num_traits;

use std::cmp::PartialEq;
use std::collections::HashMap;
use num_bigint::BigInt;
use num_traits::{ToPrimitive, Zero};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ALUOperations {
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulo,
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
    Return
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PotatoCodes {
    MovRegisterToStack(Registers, usize),
    MovStackToRegister(usize, Registers),
    Operate(ALUOperations),
    DataValue(BigInt),
    // move instruction data value to register
    MovDataValueToRegister(usize, Registers),
    // resize value in output register to fit in stack
    ResizeOutput(usize)
}

#[derive(Clone, Debug)]
pub struct StepResult {
    pub halted: bool,
    pub time_steps: usize,
    pub return_value: Option<BigInt>
}

#[derive(Clone, Debug)]
pub struct PotatoCPUSpec {
    pub num_scratch_registers: u8,
    // TODO: add stack width
}

pub struct PotatoCPU {
    pub stack: Vec<u32>,
    pub instructions: Vec<PotatoCodes>,
    pub time_steps: usize,
    pub program_counter: usize,
    pub num_scratch_registers: u8,
    pub registers: HashMap<Registers, BigInt>,
    pub halted: bool
}

impl PotatoCPU {
    pub fn new(instructions: Vec<PotatoCodes>) -> PotatoCPU {
        PotatoCPU {
            stack: vec![],
            instructions,
            time_steps: 0,
            program_counter: 0,
            num_scratch_registers: 2,
            registers: HashMap::new(),
            halted: false
        }
    }

    pub fn step(&mut self) -> StepResult {
        if self.halted {
            return StepResult {
                halted: true,
                time_steps: self.time_steps,
                return_value: self.registers.get(&Registers::Return).cloned()
            };
        }

        self.time_steps += 1;
        self.program_counter += 1;
        if self.program_counter >= self.instructions.len() {
            self.halted = true;
        }

        let instruction = &self.instructions[self.program_counter];
        match instruction {
            PotatoCodes::MovRegisterToStack(reg, index) => {
                // TODO: add instruction to copy register to multiple stack addresses
                //  (so that the whole register value can be copied)
                let value = self.registers.get(reg).cloned().unwrap_or(BigInt::from(0));
                if *index >= self.stack.len() {
                    self.stack.resize(index + 1, 0);
                }
                // TODO: dynamic stack width support
                self.stack[*index] = value.to_u32().unwrap_or(0);
            },
            PotatoCodes::MovStackToRegister(index, reg) => {
                let value = if *index < self.stack.len() {
                    BigInt::from(self.stack[*index])
                } else {
                    BigInt::from(0)
                };
                self.registers.insert(reg.clone(), value);
            },
            PotatoCodes::Operate(op) => {
                let a = self.registers.get(&Registers::InputA).cloned().unwrap_or(BigInt::from(0));
                let b = self.registers.get(&Registers::InputB).cloned().unwrap_or(BigInt::from(0));
                let result = match op {
                    ALUOperations::Add => a + b,
                    ALUOperations::Subtract => a - b,
                    ALUOperations::Multiply => a * b,
                    ALUOperations::Divide => {
                        if b.is_zero() {
                            BigInt::from(0)
                        } else {
                            a / b
                        }
                    },
                    ALUOperations::Modulo => {
                        if b.is_zero() {
                            BigInt::from(0)
                        } else {
                            a % b
                        }
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
            return_value: self.registers.get(&Registers::Return).cloned()
        }
    }
}