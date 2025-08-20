use crate::parser::parse::{parse_from_filepath, ASTFunction, ASTProgram, Expression, ExpressionVariant, Statement, SupportedUnaryOperators};
use crate::parser::parser_helpers::{ParseError, PoppedTokenContext};
use crate::tacky::tacky_symbols::{TackyFunction, TackyInstruction, TackyProgram, TackyValue, TackyVariable};

const TAB: &str = "    ";

pub trait AsmSymbol {
    fn to_asm_code(self) -> String;
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
    fn to_asm_code(self) -> String {
        let mut code = self.function.to_asm_code();
        code.push_str(".section .note.GNU-stack,\"\",@progbits\n");
        code
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
    pub fn to_stack_allocated(
        &self, stack_value: u64, offset_size: u64
    ) {
        for instruction in &self.instructions {

        }
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
    fn to_asm_code(self) -> String {
        let mut code = "".to_string();

        code.push_str(&format!("{TAB}.globl {}\n", self.name));
        code.push_str(&*self.contexts_to_string());
        code.push_str(&format!("{}:\n", self.name));

        for instruction in self.instructions {
            let inner_code = &instruction.to_asm_code();
            let indented_inner_code = indent::indent_all_with(TAB, inner_code);
            // println!("Indented inner code: {}", indented_inner_code);
            code.push_str(&*indented_inner_code);
        }
        code
    }
}

#[derive(Clone, Debug)]
pub enum Register {
    EAX,
    R10D
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
    pub fn to_stack_allocated(
        self, stack_value: u64, offset_size: u64
    ) -> StackAllocation {

    }
}

#[derive(Clone, Debug)]
pub struct AsmUnaryInstruction {
    operator: SupportedUnaryOperators,
    operand: AsmOperand,
}

#[derive(Clone, Debug)]
pub enum AsmInstruction {
    Mov(MovInstruction),
    Unary(AsmUnaryInstruction),
    Ret,
}
impl AsmSymbol for AsmInstruction {
    fn to_asm_code(self) -> String {
        match self {
            AsmInstruction::Mov(mov_instruction) => mov_instruction.to_asm_code(),
            AsmInstruction::Ret => "ret\n".to_string(),
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
    pub fn to_stack_allocated(
        &self, stack_value: u64, offset_size: u64
    ) -> Self {
        match self {
            AsmInstruction::Mov(mov_instruction) => {
                let src = mov_instruction.source.clone();
                let dst = AsmOperand::Stack(stack_size);
                AsmInstruction::Mov(MovInstruction::new(src, dst))
            },
            AsmInstruction::Ret => {
                // Ret instruction does not need to be stack allocated
                self.clone()
            },
            _ => panic!("Unsupported instruction for stack allocation"),
        }
    })
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
    fn to_asm_code(self) -> String {
        format!(
            "mov {}, {}\n",
            self.source.to_asm_code(),
            self.destination.to_asm_code()
        )
    }
}

#[derive(Clone, Debug)]
pub struct StackAllocation {
    pub(crate) offset: u64,
    pub(crate) pop_contexts: Vec<PoppedTokenContext>,
    pub(crate) tacky_var: Option<TackyVariable>,
}
impl StackAllocation {
    pub fn from_pseudo_register(
        pseudo_register: PseudoRegister, stack_value: u64
    ) -> Self {
        let mut stack_allocation = StackAllocation {
            offset: stack_value,
            pop_contexts: pseudo_register.pop_contexts.clone(),
            tacky_var: pseudo_register.tacky_var.clone(),
        };
        stack_allocation
    }
}

#[derive(Clone, Debug)]
pub enum AsmOperand {
    ImmediateValue(AsmImmediateValue),
    Register(Register),
    Pseudo(PseudoRegister),
    Stack(StackAllocation)
}
impl AsmSymbol for AsmOperand {
    fn to_asm_code(self) -> String {
        match self {
            AsmOperand::ImmediateValue(value) => value.to_asm_code(),
            AsmOperand::Register => "%eax".to_string(),
        }
    }
}
impl AsmOperand {
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
    pub fn to_stack_allocated(
        self, stack_value: u64
    ) -> (Self, bool) {
        /*
        Converts the AsmOperand to a stack allocation if it is a pseudo register.
        returns a tuple containing the new AsmOperand and a boolean indicating
        whether it was converted to a stack allocation.
        */
        match self {
            AsmOperand::Pseudo(pseudo_register) => {
                let allocation = StackAllocation::from_pseudo_register(
                    pseudo_register, stack_value
                );
                let new_instruction = AsmOperand::Stack(allocation);
                (new_instruction, true)
            },
            other => {
                (other.clone(), false)
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
    fn to_asm_code(self) -> String {
        format!("${}", self.value)
    }
}


pub fn asm_gen_from_filepath(
    file_path: &str, verbose: bool
) -> Result<AsmProgram, ParseError> {
    let parse_result = parse_from_filepath(file_path, verbose);
    let program = match parse_result {
        Ok(program) => program,
        Err(err) => return Err(err),
    };

    let asm_program = AsmProgram::from_ast_program(program);
    Ok(asm_program)
}
