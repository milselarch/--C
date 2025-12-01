use crate::asm_gen::asm_symbols::AsmSymbol;
use crate::asm_gen::asm_symbols::AsmOperand;
use crate::parser::parse::Identifier;

#[derive(Clone, Debug)]
pub enum ConditionalCompareTypes {
    Equal,
    NotEqual,
    GreaterThan,
    GreaterThanOrEqual,
    LessThan,
    LessThanOrEqual,
}

#[derive(Clone, Debug)]
pub struct AsmCompareInstruction {
    // Cmp in the textbook
    pub left: AsmOperand,
    pub right: AsmOperand,
}
impl AsmCompareInstruction {
    pub fn new(left: AsmOperand, right: AsmOperand) -> Self {
        AsmCompareInstruction { left, right }
    }
}
impl AsmSymbol for AsmCompareInstruction {
    fn to_asm_code(self) -> Result<String, crate::asm_gen::asm_symbols::AsmGenError> {
        // TODO: handle stack allocation like for mov
        let left_asm = self.left.to_asm_code()?;
        let right_asm = self.right.to_asm_code()?;
        Ok(format!("cmp {}, {}", left_asm, right_asm))
    }
}
#[derive(Clone, Debug)]
pub struct AsmJumpConditionalInstruction {
    // JmpCC in the textbook
    identifier: Identifier,
    condition: ConditionalCompareTypes
}
impl AsmJumpConditionalInstruction {
    pub fn new(
        identifier: Identifier,
        condition: ConditionalCompareTypes
    ) -> Self {
        AsmJumpConditionalInstruction {
            identifier,
            condition
        }
    }
}
#[derive(Clone, Debug)]
pub struct AsmSetConditionalInstruction {
    // SetCC in the textbook
    pub(crate) destination: AsmOperand,
    pub(crate) condition: ConditionalCompareTypes
}