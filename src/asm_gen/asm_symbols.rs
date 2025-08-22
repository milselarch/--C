use std::iter::Scan;
use crate::parser::parse::{
    parse_from_filepath, Expression, ExpressionVariant,
    Statement, SupportedUnaryOperators
};
use crate::parser::parser_helpers::{ParseError, PoppedTokenContext};
use crate::tacky::tacky_symbols::{tacky_gen_from_filepath, TackyFunction, TackyInstruction, TackyProgram, TackyValue, TackyVariable};

const TAB: &str = "    ";
const SCRATCH_REGISTER: &str = "%r10d";
const STACK_REGISTER: &str = "%rsp";
// base of current stack frame
const BASE_REGISTER: &str = "%rbp";


#[derive(Debug)]
pub enum AsmGenError {
    InvalidInstructionType(String),
    UnsupportedInstruction(String),
    ParseError(ParseError)
}

pub trait AsmSymbol {
    fn to_asm_code(self) -> Result<String, AsmGenError>;
}
pub trait HasPopContexts: Clone {
    fn _get_pop_contexts(&self) -> &Vec<PoppedTokenContext>;
    fn _add_pop_context(&mut self, pop_context: PoppedTokenContext);
    fn _add_pop_context_opt(&mut self, pop_context: Option<PoppedTokenContext>) {
        if let Some(pop_context) = pop_context {
            self._add_pop_context(pop_context);
        }
    }
    fn with_added_pop_context(self, pop_context: Option<PoppedTokenContext>) -> Self {
        let mut new = self.clone();
        new._add_pop_context_opt(pop_context);
        new
    }

    fn contexts_to_string(&self) -> String {
        let contexts = self._get_pop_contexts();
        contexts.iter().map(|c| {
            format!(
                "// TOKEN_RANGE[{}, {}], SOURCE_RANGE[{}, {}]",
                c.start_token_position, c.end_token_position,
                c.start_source_position, c.end_source_position
            )
        }).collect::<Vec<String>>().join("\n") + "\n"
    }
}
pub trait ToStackAllocated {
    fn to_stack_allocated(
        &self, stack_value: u64, offset_size: u64
        // returns a tuple of (Self, new stack_value)
    ) -> (Self, u64) where Self: Sized;
}

pub struct AsmProgram {
    pub(crate) function: AsmFunction,
}
impl AsmProgram {
    pub fn new(function: AsmFunction) -> AsmProgram {
        AsmProgram { function }
    }
    pub fn from_tacky_program(
        tacky_program: TackyProgram
    ) -> Self {
        Self::new(AsmFunction::from_tacky_function(tacky_program.function))
    }
}
impl AsmSymbol for AsmProgram {
    fn to_asm_code(self) -> Result<String, AsmGenError> {
        let mut code = self.function.to_asm_code()?;
        code.push_str(".section .note.GNU-stack,\"\",@progbits\n");
        Ok(code)
    }
}

#[derive(Clone, Debug)]
pub struct AsmFunction {
    pub(crate) name: String,
    pub(crate) instructions: Vec<AsmInstruction>,
    pub(crate) pop_contexts: Vec<PoppedTokenContext>,
}
impl AsmFunction {
    pub fn new(name: String) -> AsmFunction {
        AsmFunction {
            name,
            instructions: vec![],
            pop_contexts: vec![],
        }
    }
    pub fn add_instruction(&mut self, instruction: AsmInstruction) {
        self.instructions.push(instruction);
    }
    pub fn add_instructions(
        mut self, instructions: Vec<AsmInstruction>
    ) -> AsmFunction {
        self.instructions = instructions;
        self
    }
    pub fn from_tacky_function(
        tacky_function: TackyFunction
    ) -> AsmFunction {
        let mut asm_function = AsmFunction::new(tacky_function.name_to_string());
        for tacky_instruction in tacky_function.instructions {
            let asm_instructions =
                AsmInstruction::from_tacky_instruction(tacky_instruction);
            asm_function.instructions.extend(asm_instructions);
        }
        asm_function
    }
}
impl HasPopContexts for AsmFunction {
    fn _get_pop_contexts(&self) -> &Vec<PoppedTokenContext> {
        &self.pop_contexts
    }
    fn _add_pop_context(&mut self, pop_context: PoppedTokenContext) {
        self.pop_contexts.push(pop_context);
    }
}
impl AsmSymbol for AsmFunction {
    fn to_asm_code(self) -> Result<String, AsmGenError> {
        let mut code = "".to_string();

        code.push_str(&format!("{TAB}.globl {}\n", self.name));
        code.push_str(&*self.contexts_to_string());
        code.push_str(&format!("{}:\n", self.name));

        for instruction in self.instructions {
            let inner_code = &instruction.to_asm_code()?;
            let indented_inner_code = indent::indent_all_with(TAB, inner_code);
            // println!("Indented inner code: {}", indented_inner_code);
            code.push_str(&*indented_inner_code);
        }
        Ok(code)
    }
}
impl ToStackAllocated for AsmFunction {
    fn to_stack_allocated(
        &self, stack_value: u64, offset_size: u64
    ) -> (Self, u64) {
        let mut new_instructions = vec![];
        let mut new_stack_value = stack_value;

        for instruction in &self.instructions {
            let (new_instruction, updated_stack_value) =
                instruction.to_stack_allocated(new_stack_value, offset_size);
            new_instructions.push(new_instruction);
            new_stack_value = updated_stack_value;
        }

        let new_function = AsmFunction {
            name: self.name.clone(),
            instructions: new_instructions,
            pop_contexts: self.pop_contexts.clone(),
        };
        (new_function, new_stack_value)
    }
}

#[derive(Clone, Debug)]
pub enum Register {
    EAX,
    R10D
}
impl AsmSymbol for Register {
    fn to_asm_code(self) -> Result<String, AsmGenError> {
        match self {
            Register::EAX => Ok("%eax".to_string()),
            Register::R10D => Ok("%r10d".to_string()),
        }
    }
}

#[derive(Debug, Clone)]
pub struct PseudoRegister {
    pub(crate) id: u64,
    pub(crate) name: String,
    pub(crate) pop_contexts: Vec<PoppedTokenContext>,
    pub(crate) tacky_var: Option<TackyVariable>,
}
impl PartialEq for PseudoRegister {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}
impl HasPopContexts for PseudoRegister {
    fn _get_pop_contexts(&self) -> &Vec<PoppedTokenContext> {
        &self.pop_contexts
    }
    fn _add_pop_context(&mut self, pop_context: PoppedTokenContext) {
        self.pop_contexts.push(pop_context);
    }
}
impl PseudoRegister {
    pub fn new(id: u64, name: String) -> PseudoRegister {
        PseudoRegister {
            id,
            name,
            pop_contexts: vec![],
            tacky_var: None,
        }
    }
    pub fn set_tacky_var(
        &mut self, tacky_var: TackyVariable
    ) {
        self.tacky_var = Some(tacky_var);
    }
    pub fn with_tacky_var_context(
        mut self, tacky_var: TackyVariable
    ) -> PseudoRegister {
        let mut cloned = self.clone();
        cloned.set_tacky_var(tacky_var);
        cloned
    }

    pub fn from_tacky_var(tacky_var: TackyVariable) -> PseudoRegister {
        let cloned_var = tacky_var.clone();
        let mut pseudo_register = PseudoRegister::new(tacky_var.id, tacky_var.name);
        pseudo_register.set_tacky_var(cloned_var);
        pseudo_register
    }
    pub fn to_stack_allocation(
        self, stack_value: u64, offset_size: u64
    ) -> StackAddress {
        StackAddress::from_pseudo_register(
            &self, stack_value, offset_size
        )
    }
}

#[derive(Clone, Debug)]
pub struct AsmUnaryInstruction {
    operator: SupportedUnaryOperators,
    operand: AsmOperand,
}
impl ToStackAllocated for AsmUnaryInstruction {
    fn to_stack_allocated(
        &self, stack_value: u64, offset_size: u64
    ) -> (Self, u64) {
        let (operand, new_stack_value) =
            self.operand.to_stack_allocated(stack_value, offset_size);
        let new_instruction = AsmUnaryInstruction {
            operator: self.operator.clone(),
            operand,
        };
        (new_instruction, new_stack_value)
    }
}
impl AsmSymbol for AsmUnaryInstruction {
    fn to_asm_code(self) -> Result<String, AsmGenError> {
        let operand_asm = self.operand.to_asm_code()?;
        match self.operator {
            SupportedUnaryOperators::Minus => {
                Ok(format!("negl {}\n", operand_asm))
            },
            SupportedUnaryOperators::BitwiseNot => {
                Ok(format!("notl {}\n", operand_asm))
            },
            _ => Err(AsmGenError::UnsupportedInstruction(
                format!("Unsupported unary operator: {:?}", self.operator)
            )),
        }
    }
}

#[derive(Clone, Debug)]
pub enum AsmInstruction {
    Mov(MovInstruction),
    Unary(AsmUnaryInstruction),
    AllocateStack(StackAllocation),
    Ret,
}
impl AsmSymbol for AsmInstruction {
    fn to_asm_code(self) -> Result<String, AsmGenError> {
        match self {
            AsmInstruction::Mov(mov_instruction) => {
                Ok(mov_instruction.to_asm_code()?)
            },
            AsmInstruction::Unary(unary_instruction) => {
                Ok(unary_instruction.to_asm_code()?)
            },
            AsmInstruction::AllocateStack(stack_allocation) => {
                Ok(stack_allocation.to_asm_code()?)
            },
            AsmInstruction::Ret => Ok("ret\n".to_string()),
        }
    }
}
impl AsmInstruction {
    pub fn from_tacky_instruction(
        tacky_instruction: TackyInstruction
    ) -> Vec<Self> {
        match tacky_instruction {
            TackyInstruction::UnaryInstruction(unary_instruction) => {
                todo!()
            },
            TackyInstruction::Return(tacky_value) => {
                let src_operand = match tacky_value {
                    TackyValue::Constant(ast_constant) => {
                        let value = ast_constant.to_u64().unwrap();
                        let asm_value = AsmImmediateValue::new(value)
                            .with_added_pop_context(ast_constant.pop_context.clone());
                        AsmOperand::ImmediateValue(asm_value)
                    },
                    TackyValue::Var(tacky_var) => {
                        // Handle variable return case
                        AsmOperand::Pseudo(PseudoRegister::from_tacky_var(tacky_var))
                    },
                };
                let dst_operand = AsmOperand::Register(Register::EAX);
                let mov_instruction = MovInstruction::new(src_operand, dst_operand);
                vec![
                    AsmInstruction::Mov(mov_instruction),
                    AsmInstruction::Ret
                ]
            },
        }
    }
}
impl ToStackAllocated for AsmInstruction {
    fn to_stack_allocated(
        &self, stack_value: u64, offset_size: u64
    ) -> (Self, u64) {
        match self {
            AsmInstruction::Mov(mov_instruction) => {
                let (new_mov_instruction, new_stack_value) =
                    mov_instruction.to_stack_allocated(stack_value, offset_size);
                (AsmInstruction::Mov(new_mov_instruction), new_stack_value)
            },
            AsmInstruction::Unary(unary_instruction) => {
                let (new_unary_instruction, new_stack_value) =
                    unary_instruction.to_stack_allocated(stack_value, offset_size);
                (AsmInstruction::Unary(new_unary_instruction), new_stack_value)
            },
            others => {
                // For other instructions, we assume they do not require stack allocation
                (others.clone(), stack_value)
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct MovInstruction {
    pub(crate) source: AsmOperand,
    pub(crate) destination: AsmOperand,
}
impl MovInstruction {
    pub fn new(source: AsmOperand, destination: AsmOperand) -> Self {
        MovInstruction { source, destination }
    }
}
impl AsmSymbol for MovInstruction {
    fn to_asm_code(self) -> Result<String, AsmGenError> {
        let is_src_stack_alloc = self.source.is_stack_alloc();
        let is_dst_stack_alloc = self.destination.is_stack_alloc();
        let src_asm = self.source.to_asm_code()?;
        let dst_asm = self.destination.to_asm_code()?;

        if is_src_stack_alloc && is_dst_stack_alloc {
            let mut asm_code: String = String::new();
            asm_code.push_str(&format!("movl {src_asm}, {SCRATCH_REGISTER}\n"));
            asm_code.push_str(&format!("movl {SCRATCH_REGISTER}, {dst_asm}"));
            return Ok(asm_code);
        } else {
            Ok(format!("mov {}, {}\n", src_asm, dst_asm))
        }
    }
}
impl ToStackAllocated for MovInstruction {
    fn to_stack_allocated(
        &self, stack_value: u64, offset_size: u64
    ) -> (Self, u64) {
        let (source, stack_value) =
            self.source.to_stack_allocated(stack_value, offset_size);
        let (destination, stack_value) =
            self.destination.to_stack_allocated(stack_value, offset_size);

        let new_instruction = MovInstruction { source, destination };
        (new_instruction, stack_value)
    }
}

#[derive(Clone, Debug)]
pub struct StackAllocation {
    pub(crate) offset: u64,
    pub(crate) offset_size: u64,
    pub(crate) pop_contexts: Vec<PoppedTokenContext>,
    pub(crate) tacky_var: Option<TackyVariable>,
}
impl AsmSymbol for StackAllocation {
    fn to_asm_code(self) -> Result<String, AsmGenError> {
        Ok(format!("subq ${}, {STACK_REGISTER}", self.offset))
    }
}

#[derive(Clone, Debug)]
pub struct StackAddress {
    pub(crate) offset: u64,
    pub(crate) offset_size: u64,
    pub(crate) pop_contexts: Vec<PoppedTokenContext>,
    pub(crate) tacky_var: Option<TackyVariable>,
}
impl StackAddress {
    pub fn new(offset: u64, offset_size: u64) -> Self {
        StackAddress {
            offset, offset_size, pop_contexts: vec![],
            tacky_var: None
        }
    }
    pub fn from_pseudo_register(
        pseudo_register: &PseudoRegister, stack_value: u64,
        offset_size: u64
    ) -> Self {
        let mut stack_address = StackAddress {
            offset: stack_value,
            offset_size,
            pop_contexts: pseudo_register.pop_contexts.clone(),
            tacky_var: pseudo_register.tacky_var.clone(),
        };
        stack_address
    }
}
impl AsmSymbol for StackAddress {
    fn to_asm_code(self) -> Result<String, AsmGenError> {
        Ok(format!("${}{BASE_REGISTER}", self.offset))
    }
}


#[derive(Clone, Debug)]
pub enum AsmOperand {
    ImmediateValue(AsmImmediateValue),
    Register(Register),
    Pseudo(PseudoRegister),
    Stack(StackAddress)
}
impl AsmSymbol for AsmOperand {
    fn to_asm_code(self) -> Result<String, AsmGenError> {
        match self {
            AsmOperand::ImmediateValue(value) => {
                Ok(value.value.to_string())
            },
            AsmOperand::Register(register) => {
                Ok(register.to_asm_code()?)
            },
            AsmOperand::Pseudo(pseudo_register) => {
                Err(AsmGenError::InvalidInstructionType(
                    format!(
                        "Pseudo register {} not supported in assembly",
                        pseudo_register.name
                    )
                ))
            },
            AsmOperand::Stack(stack_address) => {
                Ok(stack_address.to_asm_code()?)
            }
        }
    }
}
impl AsmOperand {
    pub fn is_stack_alloc(&self) -> bool {
        matches!(self, AsmOperand::Stack(_))
    }
    pub fn from_tacky_value(tacky_value: TackyValue) -> Self {
        match tacky_value {
            TackyValue::Constant(ast_constant) => {
                let value = ast_constant.to_u64().unwrap();
                AsmOperand::ImmediateValue(AsmImmediateValue::new(value)
                    .with_added_pop_context(ast_constant.pop_context.clone()))
            },
            TackyValue::Var(tacky_var) => {
                AsmOperand::Pseudo(PseudoRegister::from_tacky_var(tacky_var))
            },
        }
    }
}
impl ToStackAllocated for AsmOperand {
    fn to_stack_allocated(
        &self, stack_value: u64, offset_size: u64
    ) -> (Self, u64) {
        /*
        Converts the AsmOperand to a stack allocation if it is a pseudo register.
        returns a tuple containing the new AsmOperand and a boolean indicating
        whether it was converted to a stack allocation.
        */
        match self {
            AsmOperand::Pseudo(pseudo_register) => {
                let stack_address = StackAddress::from_pseudo_register(
                    pseudo_register, stack_value, offset_size
                );
                let new_instruction = AsmOperand::Stack(stack_address);
                (new_instruction, stack_value + offset_size)
            },
            other => {
                (other.clone(), stack_value)
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct AsmImmediateValue {
    pub(crate) value: u64,
    pub(crate) pop_contexts: Vec<PoppedTokenContext>
}
impl AsmImmediateValue {
    pub fn new(value: u64) -> AsmImmediateValue {
        AsmImmediateValue {
            value,
            pop_contexts: vec![]
        }
    }

    pub fn from_expression(expr: Expression) -> Self {
        match expr.expr_item {
            ExpressionVariant::Constant(ref constant) => {
                let value = constant.to_u64().unwrap();
                AsmImmediateValue::new(value).with_added_pop_context(
                    expr.pop_context.clone()
                )
            },
            ExpressionVariant::UnaryOperation(_, _) => {
                panic!("Unary operations not implemented yet");
            },
        }
    }

    pub fn from_statement(statement: Statement) -> Self {
        Self::from_expression(statement.expression)
    }
}
impl HasPopContexts for AsmImmediateValue {
    fn _get_pop_contexts(&self) -> &Vec<PoppedTokenContext> {
        &self.pop_contexts
    }
    fn _add_pop_context(&mut self, pop_context: PoppedTokenContext) {
        self.pop_contexts.push(pop_context);
    }
}
impl AsmSymbol for AsmImmediateValue {
    fn to_asm_code(self) -> Result<String, AsmGenError> {
        Ok(format!("${}", self.value))
    }
}


pub fn asm_gen_from_filepath(
    file_path: &str, verbose: bool
) -> Result<AsmProgram, ParseError> {
    let tacky_program = tacky_gen_from_filepath(file_path, verbose)?;
    let asm_program = AsmProgram::from_tacky_program(tacky_program);
    Ok(asm_program)
}
