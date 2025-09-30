use crate::asm_gen::asm_symbols::{AsmGenError, AsmOperand, AsmSymbol};
use crate::asm_gen::helpers::{DiffableHashMap, StackAllocationResult, ToStackAllocated};
use crate::parser::parse::SupportedUnaryOperators;

#[derive(Clone, Debug)]
pub struct AsmUnaryInstruction {
    pub(crate) operator: SupportedUnaryOperators,
    pub(crate) destination: AsmOperand,
}
impl AsmUnaryInstruction {
    fn operator_to_asm_string(
        operator: SupportedUnaryOperators
    ) -> Result<String, AsmGenError> {
        match operator {
            SupportedUnaryOperators::Subtract => Ok("negl".to_string()),
            SupportedUnaryOperators::BitwiseNot => Ok("notl".to_string()),
            _ => Err(AsmGenError::UnsupportedInstruction(
                format!("Unsupported unary operator: {:?}", operator)
            )),
        }
    }
}
impl ToStackAllocated for AsmUnaryInstruction {
    fn to_stack_allocated(
        &self, stack_value: u64,
        allocations: &dyn DiffableHashMap<u64, u64>
    ) -> (Self, StackAllocationResult) {
        let (operand, alloc_result) =
            self.destination.to_stack_allocated(stack_value, allocations);
        let new_instruction = AsmUnaryInstruction {
            operator: self.operator.clone(),
            destination: operand,
        };
        (new_instruction, alloc_result)
    }
}
impl AsmSymbol for AsmUnaryInstruction {
    fn to_asm_code(self) -> Result<String, AsmGenError> {
        let operand_asm = self.destination.to_asm_code()?;
        let operator_asm = Self::operator_to_asm_string(self.operator)?;
        Ok(format!("{} {}", operator_asm, operand_asm))
    }
}