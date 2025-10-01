use crate::asm_gen::asm_symbols::{
    AsmGenError, AsmInstruction, AsmOperand, AsmSymbol,
    MovInstruction, Register
};
use crate::asm_gen::helpers::{
    BufferedHashMap, DiffableHashMap, StackAllocationResult,
    ToStackAllocated
};
use crate::asm_gen::interger_division::AsmIntegerDivision;
use crate::parser::parse::SupportedBinaryOperators;
use crate::tacky::tacky_symbols::{BinaryInstruction, TackyValue};

#[derive(Clone, Debug)]
pub enum AsmBinaryOperators {
    Add,
    Subtract,
    Multiply
}
impl AsmBinaryOperators {
    pub fn to_asm_string(&self) -> String {
        match self {
            AsmBinaryOperators::Add => "addl".to_string(),
            AsmBinaryOperators::Subtract => "subl".to_string(),
            AsmBinaryOperators::Multiply => "imull".to_string(),
        }
    }
    pub fn from_supported(op: SupportedBinaryOperators) -> Result<Self, AsmGenError> {
        match op {
            SupportedBinaryOperators::Add => Ok(AsmBinaryOperators::Add),
            SupportedBinaryOperators::Subtract => Ok(AsmBinaryOperators::Subtract),
            SupportedBinaryOperators::Multiply => Ok(AsmBinaryOperators::Multiply),
            _ => Err(AsmGenError::UnsupportedInstruction(
                format!("Unsupported binary operator: {:?}", op))
            ),
        }
    }
}

#[derive(Clone, Debug)]
pub enum DivisionOutputs {
    Quotient,
    Remainder
}

#[derive(Clone, Debug)]
pub struct AsmBinaryInstruction {
    pub(crate) operator: AsmBinaryOperators,
    pub(crate) source: AsmOperand,
    pub(crate) destination: AsmOperand,
}
impl AsmBinaryInstruction {
    pub fn build_divide_instructions(
        left_operand: AsmOperand,
        right_operand: AsmOperand,
        dst_operand: AsmOperand,
        desired_output: DivisionOutputs
    ) -> Vec<AsmInstruction> {
        // Move left operand into EAX (division input register)
        let move_into_instruction = MovInstruction::new(
            left_operand.clone(), AsmOperand::Register(Register::EAX)
        );
        let output_register = match desired_output {
            DivisionOutputs::Quotient => AsmOperand::Register(Register::EAX),
            DivisionOutputs::Remainder => AsmOperand::Register(Register::EDX),
        };
        // move division output into dst operand
        let move_out_instruction = MovInstruction::new(
            output_register, dst_operand.clone()
        );
        vec![
            AsmInstruction::Mov(move_into_instruction),
            AsmInstruction::SignExtension,
            AsmInstruction::IntegerDivision(
                AsmIntegerDivision::new(right_operand.clone())
            ),
            AsmInstruction::Mov(move_out_instruction)
        ]
    }

    pub fn unpack_from_tacky(binary_instruction: BinaryInstruction) -> Vec<AsmInstruction> {
        /*
      TACKY:
      ----------------------------
      Binary(op, src1, src2, dst)
      ----------------------------
      ASM:
      ----------------------------
      Mov(src1, dst)
      Binary(op, src2, dst)

      ASM instruction applies op to dst using src2
      and stores result in dst
      */
        let left_operand = AsmOperand::from_tacky_value(binary_instruction.left);
        let right_operand = AsmOperand::from_tacky_value(binary_instruction.right.clone());
        let dst_operand = AsmOperand::from_tacky_value(
            TackyValue::Var(binary_instruction.dst)
        );

        match binary_instruction.operator {
            SupportedBinaryOperators::Divide => {
                return Self::build_divide_instructions(
                    left_operand, right_operand, dst_operand,
                    DivisionOutputs::Quotient
                );
            }
            SupportedBinaryOperators::Modulo => {
                return Self::build_divide_instructions(
                    left_operand, right_operand, dst_operand,
                    DivisionOutputs::Remainder
                );
            },
            _ => {}
        }

        let asm_binary_operator = AsmBinaryOperators::from_supported(
            binary_instruction.operator
        ).unwrap();
        let asm_mov_instruction = MovInstruction::new(
            left_operand.clone(), dst_operand.clone()
        );

        let asm_binary_instruction = AsmBinaryInstruction {
            operator: asm_binary_operator,
            source: right_operand,
            destination: dst_operand
        };
        vec![
            AsmInstruction::Mov(asm_mov_instruction),
            AsmInstruction::Binary(asm_binary_instruction)
        ]
    }
}
impl ToStackAllocated for AsmBinaryInstruction {
    fn to_stack_allocated(
        &self, stack_value: u64,
        allocations: &dyn DiffableHashMap<u64, u64>
    ) -> (Self, StackAllocationResult) {
        let mut alloc_buffer = BufferedHashMap::new(allocations);

        let (source, src_alloc_result) =
            self.source.to_stack_allocated(stack_value, allocations);
        let stack_value = src_alloc_result.new_stack_value;
        alloc_buffer.apply_changes(src_alloc_result.new_stack_allocations).unwrap();

        let (destination, dest_alloc) =
            self.destination.to_stack_allocated(stack_value, allocations);
        let stack_value = dest_alloc.new_stack_value;
        alloc_buffer.apply_changes(dest_alloc.new_stack_allocations).unwrap();

        let new_instruction = AsmBinaryInstruction {
            operator: self.operator.clone(),
            source,
            destination,
        };
        let alloc_result =
            StackAllocationResult::new_from_buffered(stack_value, alloc_buffer);
        (new_instruction, alloc_result)
    }
}
impl AsmSymbol for AsmBinaryInstruction {
    fn to_asm_code(self) -> Result<String, AsmGenError> {
        let operator_asm = self.operator.to_asm_string();
        let source_asm = self.source.to_asm_code()?;
        let destination_asm = self.destination.to_asm_code()?;
        Ok(format!("{} {}, {}", operator_asm, source_asm, destination_asm))
    }
}
