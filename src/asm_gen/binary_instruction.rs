use crate::asm_gen::asm_symbols::AsmOperand;
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
