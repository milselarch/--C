use crate::asm_gen::asm_symbols::{AsmOperand, AsmSymbol};
use crate::asm_gen::helpers::{BufferedHashMap, DiffableHashMap, StackAllocationResult, ToStackAllocated};
use crate::parser::parse::SupportedBinaryOperators;

#[derive(Clone, Debug)]
pub enum AsmBinaryOperators {
    Add,
    Subtract,
    Multiply
}
impl AsmBinaryOperators {
    pub fn to_asm_string(&self) -> Result<String, String> {
        match self {
            AsmBinaryOperators::Add => Ok("addl".to_string()),
            AsmBinaryOperators::Subtract => Ok("subl".to_string()),
            AsmBinaryOperators::Multiply => Ok("imull".to_string()),
        }
    }
}

#[derive(Clone, Debug)]
pub struct AsmBinaryInstruction {
    operator: AsmBinaryOperators,
    source: AsmOperand,
    destination: AsmOperand,
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
    fn to_asm_code(self) -> Result<String, String> {
        let operator_asm = self.operator.to_asm_string()?;
        let source_asm = self.source.to_asm_code()?;
        let destination_asm = self.destination.to_asm_code()?;
        Ok(format!("{} {}, {}", operator_asm, source_asm, destination_asm))
    }
}
