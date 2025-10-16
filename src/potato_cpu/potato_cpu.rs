extern crate num_bigint;
extern crate num_traits;

use crate::potato_cpu::bit_allocation::{
    BitAllocation, FixedBitAllocation, GrowableBitAllocation
};
use arbitrary_int::{u4, UInt};
use strum::IntoEnumIterator;
use std::cmp::{Ordering, PartialEq, PartialOrd};
use std::collections::HashMap;
use num_traits::{ToPrimitive, Zero};
use strum_macros::EnumIter;

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
    // O(n), assembly implementation would be O(n^2)
    ShiftLeft,
    // O(n), assembly implementation would be O(n^2)
    ShiftRight,
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

#[derive(Clone, Debug, Eq, PartialEq, Hash, EnumIter)]
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
    register: Registers
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PotatoCodes {
    // register, stack address
    MovRegisterToStack(Registers, usize),
    // stack address, num stack addresses to copy, register
    MovStackToRegister(MovStackToRegister),
    CopyRegisterToRegister(Registers, Registers),

    /*
    START
    VAL_0[0] STOP_0[0] VAL_1[0] STOP_1[0] ... VAL_n[0] STOP_n[0]
    VAL_0[1] STOP_0[1] VAL_1[1] STOP_1[1] ... VAL_n[1] STOP_n[1]
    ...
    */
    StrideMovRegisterToStack(StrideMovRegisterToStack),
    StrideMovStackToRegister(StrideMovStackToRegister),

    Operate(ALUOperations),
    DataValue(GrowableBitAllocation),
    // move instruction data value to register
    MovDataValueToRegister(usize, Registers),
    // jump to instruction index if Registers::Output is zero
    JumpIfZero(usize),
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
impl PotatoSpec {
    pub fn new(
        instructions: Vec<PotatoCodes>,
        num_scratch_registers: u8,
        stack_width: u16
    ) -> PotatoSpec {
        PotatoSpec {
            instructions,
            num_scratch_registers,
            stack_width
        }
    }
    pub fn set_instructions(mut self, instructions: Vec<PotatoCodes>) -> Self {
        self.instructions = instructions;
        self
    }
    pub fn get_instructions(&self) -> &Vec<PotatoCodes> {
        &self.instructions
    }
    pub fn get_num_scratch_registers(&self) -> u8 {
        self.num_scratch_registers
    }
    pub fn get_stack_width(&self) -> u16 {
        self.stack_width
    }
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

impl PartialOrd for GrowableBitAllocation {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.to_big_num().partial_cmp(&other.to_big_num())
    }
}

impl PotatoCPU {
    pub fn new(spec: &PotatoSpec) -> PotatoCPU {
        let registers = Self::init_registers(&spec);
        PotatoCPU {
            stack: vec![],
            spec: spec.clone(),
            time_steps: 0,
            program_counter: 0,
            registers,
            halted: false
        }
    }
    pub fn set_instructions(mut self, instructions: Vec<PotatoCodes>) -> Self {
        assert_eq!(self.time_steps, 0);
        self.spec = self.spec.set_instructions(instructions);
        self
    }

    pub fn init_registers(
        spec: &PotatoSpec
    ) -> HashMap<Registers, GrowableBitAllocation> {
        let mut registers = HashMap::new();

        for register in Registers::iter() {
            let empty_val = GrowableBitAllocation::new(0);
            if let Registers::Scratch(scratch_register_no) = register {
                if scratch_register_no < spec.num_scratch_registers {
                    registers.insert(register.clone(), empty_val);
                }
            } else {
                registers.insert(register.clone(), empty_val);
            }
        }
        registers
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
    pub fn load_register(&mut self, reg: Registers) -> &mut GrowableBitAllocation {
        self.validate_register(&reg);
        self.registers.entry(reg).or_insert(
            GrowableBitAllocation::new(0)
        )
    }
    pub fn read_register(&self, reg: Registers) -> &GrowableBitAllocation {
        self.validate_register(&reg);
        self.registers.get(&reg).unwrap()
    }

    pub fn run(&mut self, max_steps: usize) -> StepResult {
        for _ in 0..max_steps {
            let step_result = self.step();
            if step_result.halted {
                return step_result;
            }
        }
        StepResult {
            halted: self.halted,
            time_steps: self.time_steps
        }
    }
    pub fn step(&mut self) -> StepResult {
        if self.halted {
            return StepResult {
                halted: true,
                time_steps: self.time_steps,
            };
        }

        let instructions = self.get_instructions();
        if self.program_counter >= instructions.len() {
            self.halted = true;
            return StepResult {
                halted: true,
                time_steps: self.time_steps
            }
        }

        let instruction = instructions[self.program_counter].clone();

        match instruction {
            PotatoCodes::MovRegisterToStack(reg, index) => {
                let register_value = self.read_register(reg);
                let chunks = register_value.split(self.spec.stack_width as usize);
                for (i, chunk) in chunks.into_iter().enumerate() {
                    self.assign_to_stack(index + i, chunk);
                }
            },
            PotatoCodes::MovStackToRegister(params) => {
                let mut chunks: Vec<FixedBitAllocation> = vec![];
                for i in 0..params.num_stack_addresses {
                    let stack_value = self.read_from_stack(params.stack_address + i);
                    chunks.push(stack_value);
                }
                let new_register_value =
                    GrowableBitAllocation::from_fixed_allocations(&chunks);
                self.registers.insert(params.register, new_register_value);
            },
            PotatoCodes::CopyRegisterToRegister(src, dst) => {
                let src_value = self.read_register(src).clone();
                self.registers.insert(dst, src_value);
            },
            PotatoCodes::StrideMovRegisterToStack(params) => {
                let register_value = self.read_register(params.register);
                let chunks = register_value.split(self.spec.stack_width as usize);
                let data_stride = params.stride * 2;
                let is_last_chunk_index = chunks.len() - 1;

                for (k, chunk) in chunks.into_iter().enumerate() {
                    // stack position where current chunk's value is written
                    let data_pos = params.start_stack_address + k * data_stride;
                    // stack position where current chunk's continue value is written
                    // this flags whether there is still more chunks after the current one
                    let data_cont_pos = data_pos + 1;
                    let is_last_chunk = k == is_last_chunk_index;

                    let mut cont_stack_value = self.spawn_new_stack_value();
                    if !is_last_chunk {
                        // flag that there is more data after this chunk index
                        cont_stack_value.set(0, true);
                    }

                    self.assign_to_stack(data_pos, chunk);
                    self.assign_to_stack(data_cont_pos, cont_stack_value);
                }
            }
            PotatoCodes::StrideMovStackToRegister(params) => {
                let data_stride = params.stride * 2;
                let mut chunks: Vec<FixedBitAllocation> = vec![];
                let chunk_index: usize = 0;

                loop {
                    let data_pos = params.start_stack_address + chunk_index * data_stride;
                    let data_cont_pos = data_pos + 1;
                    let stack_value = self.read_from_stack(data_pos);
                    chunks.push(stack_value);

                    let cont_stack_value = self.read_from_stack(data_cont_pos);
                    if !cont_stack_value.get(0) {
                        // no more data after this chunk index
                        // this is like reaching a NULL terminator in a C array
                        break;
                    }
                }

                let new_register_value =
                    GrowableBitAllocation::from_fixed_allocations(&chunks);
                self.registers.insert(params.register, new_register_value);
            },
            PotatoCodes::Operate(op) => {
                let result = self.process_alu_op(op);
                self.registers.insert(Registers::Output, result);
            },
            PotatoCodes::DataValue(..) => {
                // no-op
            }
            PotatoCodes::MovDataValueToRegister(index, reg) => {
                let instruction = &self.get_instructions()[index];
                if let PotatoCodes::DataValue(value) = instruction {
                    self.registers.insert(reg.clone(), value.clone());
                } else {
                    panic!("Expected DataValue at index {}", index)
                }
            }
            PotatoCodes::JumpIfZero(target_index) => {
                let output_value = self.read_register(Registers::Output);
                if output_value.to_big_num().is_zero() {
                    if target_index >= instructions.len() {
                        self.halted = true;
                    } else {
                        self.program_counter = target_index;
                    }
                }
            }
        }

        self.time_steps += 1;
        self.program_counter += 1;

        StepResult {
            halted: self.halted,
            time_steps: self.time_steps
        }
    }
    pub fn process_alu_op(&self, op: ALUOperations) -> GrowableBitAllocation {
        let a = self.read_register(Registers::InputA);
        let b = self.read_register(Registers::InputB);
        let a_size = a.get_length();
        let b_size = b.get_length();
        let max_size = std::cmp::max(a_size, b_size);

        let result = match op {
            ALUOperations::Add => a + b,
            ALUOperations::ReverseBits => {
                let mut cloned = a.clone();
                cloned.reverse();
                cloned
            },
            ALUOperations::BitwiseNOperation(op_code) => {
                a.apply_boolean_operation(b, op_code)
            },
            ALUOperations::ShiftLeft => {
                a << b
            },
            ALUOperations::ShiftRight => {
                a >> b
            },
            ALUOperations::CompareGreaterThan => {
                GrowableBitAllocation::new_from_bool(a > b)
            },
            ALUOperations::GetLength => {
                let length = a.get_length();
                GrowableBitAllocation::new_from_num(length)
            },
            ALUOperations::Resize => {
                let mut resized = a.clone();
                let new_size = b.to_big_num().to_usize().unwrap();
                resized.resize(new_size);
                resized
            },
            ALUOperations::ResizeModulo => {
                let mut resized_modulo = a.clone();
                let new_size = b.to_big_num().to_usize().unwrap();
                resized_modulo.resize_modulo(new_size);
                resized_modulo
            }
        };
        result
    }
}