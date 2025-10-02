use std::fmt::format;
use std::hash::{Hash, Hasher};
use crate::asm_gen::asm_symbols::TAB;
use crate::parser::parse::{
    Identifier, ASTProgram, SupportedUnaryOperators, ASTFunction, ExpressionVariant,
    ASTConstant, parse_from_filepath, SupportedBinaryOperators
};
use crate::parser::parser_helpers::{ParseError, PoppedTokenContext};


pub trait PrintableTacky {
    fn print_tacky_code(&self, depth: u64) -> String;
}

#[derive(Debug, Clone)]
pub struct TackyVariable {
    pub id: u64,
    pub name: String,
}
impl TackyVariable {
    pub fn new(id: u64) -> TackyVariable {
        TackyVariable { id, name: "".to_string() }
    }
}
impl Eq for TackyVariable {}
impl PartialEq for TackyVariable {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}
impl Hash for TackyVariable {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}
impl PrintableTacky for TackyVariable {
    fn print_tacky_code(&self, depth: u64) -> String {
        let indent = TAB.repeat(depth as usize);
        format!("{}TackyVariable: id={}, name={}\n", indent, self.id, self.name)
    }
}

#[derive(Debug, Clone)]
pub struct UnrollResult {
    pub instructions: Vec<TackyInstruction>,
    pub value: TackyValue,
    pub next_free_var_id: u64
}
impl UnrollResult {
    pub fn new(
        instructions: Vec<TackyInstruction>,
        value: TackyValue,
        next_free_var_id: u64
    ) -> UnrollResult {
        UnrollResult {
            instructions,
            value,
            next_free_var_id
        }
    }
}

#[derive(Debug, Clone)]
pub enum TackyValue {
    Constant(ASTConstant),
    Var(TackyVariable)
}
impl TackyValue {
    pub fn get_id(&self) -> Option<u64> {
        match self {
            TackyValue::Constant(_) => None,
            TackyValue::Var(v) => Some(v.id)
        }
    }
}
impl PrintableTacky for TackyValue {
    fn print_tacky_code(&self, depth: u64) -> String {
        let indent = TAB.repeat(depth as usize);
        match self {
            TackyValue::Constant(c) => {
                format!("{}Constant: {}\n", indent, c.value)
            },
            TackyValue::Var(v) => {
                format!("{}Var: id={}, name={}\n", indent, v.id, v.name)
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct UnaryInstruction {
    pub operator: SupportedUnaryOperators,
    pub src: TackyValue,
    pub dst: TackyVariable,
    pub pop_context: Option<PoppedTokenContext>
}
impl UnaryInstruction {
    pub fn to_tacky_instruction(&self) -> TackyInstruction {
        TackyInstruction::UnaryInstruction(self.clone())
    }
}
impl PrintableTacky for UnaryInstruction {
    fn print_tacky_code(&self, depth: u64) -> String {
        let indent = TAB.repeat(depth as usize);
        let mut result = String::new();
        result.push_str(&format!("{indent}UnaryInstruction:\n"));
        result.push_str(&format!(
            "{indent}{TAB}Operator: {:?}\n", self.operator
        ));
        result.push_str(&format!("{indent}{TAB}Src:\n"));
        result.push_str(&self.src.print_tacky_code(depth + 2));
        result.push_str(&format!("{indent}{TAB}Dst:\n"));
        result.push_str(&self.dst.print_tacky_code(depth + 2));
        result
    }
}

#[derive(Clone, Debug)]
pub struct BinaryInstruction {
    pub operator: SupportedBinaryOperators,
    pub left: TackyValue,
    pub right: TackyValue,
    pub dst: TackyVariable,
    pub pop_context: Option<PoppedTokenContext>
}
impl BinaryInstruction {
    pub fn to_tacky_instruction(&self) -> TackyInstruction {
        TackyInstruction::BinaryInstruction(self.clone())
    }
}
impl PrintableTacky for BinaryInstruction {
    fn print_tacky_code(&self, depth: u64) -> String {
        let indent = TAB.repeat(depth as usize);
        let mut result = String::new();
        result.push_str(&format!("{indent}BinaryInstruction:\n"));
        result.push_str(&format!(
            "{indent}{TAB}Operator: {:?}\n", self.operator
        ));
        result.push_str(&format!("{indent}{TAB}Left:\n"));
        result.push_str(&self.left.print_tacky_code(depth + 2));
        result.push_str(&format!("{indent}{TAB}Right:\n"));
        result.push_str(&self.right.print_tacky_code(depth + 2));
        result.push_str(&format!("{indent}{TAB}Dst:\n"));
        result.push_str(&self.dst.print_tacky_code(depth + 2));
        result
    }
}

#[derive(Clone, Debug)]
pub enum TackyInstruction {
    UnaryInstruction(UnaryInstruction),
    BinaryInstruction(BinaryInstruction),
    Return(TackyValue),
}
impl TackyInstruction {
    pub fn unroll_expression(
        expr_item: ExpressionVariant,
        var_counter: u64
    ) -> UnrollResult {
        match expr_item {
            ExpressionVariant::Constant(ast_constant) => {
                UnrollResult::new(
                    Vec::new(),
                    TackyValue::Constant(ast_constant.clone()),
                    var_counter
                )
            },
            ExpressionVariant::UnaryOperation(
                operator, sub_expr
            ) => {
                let sub_expr_item = sub_expr.expr_item.clone();
                let inner_unroll_res = Self::unroll_expression(
                    sub_expr_item, var_counter
                );

                let var_counter = inner_unroll_res.next_free_var_id;
                let new_var = TackyVariable::new(var_counter);
                let new_unary_instruction = UnaryInstruction {
                    operator,
                    src: inner_unroll_res.value,
                    dst: new_var.clone(),
                    pop_context: sub_expr.pop_context.clone()
                };

                let sub_instructions = inner_unroll_res.instructions;
                let mut instructions = sub_instructions.clone();
                instructions.push(new_unary_instruction.to_tacky_instruction());

                UnrollResult::new(
                    instructions,
                    TackyValue::Var(new_var),
                    var_counter + 1
                )
            }
            ExpressionVariant::BinaryOperation(operator, left, right) => {
                let left_expr_item = left.expr_item.clone();
                let right_expr_item = right.expr_item.clone();

                let left_unroll =
                    Self::unroll_expression(left_expr_item, var_counter);
                let var_counter = left_unroll.next_free_var_id;
                let right_unroll =
                    Self::unroll_expression(right_expr_item, var_counter);
                let var_counter = left_unroll.next_free_var_id;

                let new_var = TackyVariable::new(var_counter);
                let new_binary_instruction = BinaryInstruction {
                    operator,
                    left: left_unroll.value,
                    right: right_unroll.value,
                    dst: new_var.clone(),
                    pop_context: right.pop_context.clone()
                };

                let left_instructions = left_unroll.instructions;
                let right_instructions = right_unroll.instructions;
                let mut instructions = left_instructions.clone();
                instructions.extend(right_instructions.clone());
                instructions.push(new_binary_instruction.to_tacky_instruction());

                UnrollResult::new(
                    instructions,
                    TackyValue::Var(new_var),
                    var_counter + 1
                )
            }
            ExpressionVariant::ParensWrapped(sub_expr) => {
                let inner_variant = sub_expr.expr_item;
                Self::unroll_expression(inner_variant, var_counter)
            }
            _ => {
                panic!("INVALID EXPR_ITEM {}", format!("{:?}", expr_item));
            }
        }
    }
}
impl PrintableTacky for TackyInstruction {
    fn print_tacky_code(&self, depth: u64) -> String {
        match self {
            TackyInstruction::UnaryInstruction(unary) => {
                unary.print_tacky_code(depth)
            },
            TackyInstruction::BinaryInstruction(binary) => {
                binary.print_tacky_code(depth)
            },
            TackyInstruction::Return(value) => {
                let indent = TAB.repeat(depth as usize);
                let mut result = String::new();
                result.push_str(&format!("{indent}Return:\n"));
                result.push_str(&value.print_tacky_code(depth + 1));
                result
            }
        }
    }
}

pub struct TackyFunction {
    pub name: Identifier,
    pub instructions: Vec<TackyInstruction>,
    pub pop_context: Option<PoppedTokenContext>
}
impl TackyFunction {
    pub fn from_function(function: &ASTFunction) -> TackyFunction {
        let statement = &function.body;
        let expression = &statement.expression;
        let expr_item = expression.expr_item.clone();
        let inner_unroll = TackyInstruction::unroll_expression(expr_item, 0);

        let temp_value = inner_unroll.value;
        let mut sub_instructions = inner_unroll.instructions;
        let return_instruction = TackyInstruction::Return(temp_value);
        sub_instructions.push(return_instruction);

        TackyFunction {
            name: function.name.clone(),
            instructions: sub_instructions,
            pop_context: function.pop_context.clone()
        }
    }
    pub fn name_to_string(&self) -> String {
        self.name.name_to_string()
    }
}
impl PrintableTacky for TackyFunction {
    fn print_tacky_code(&self, depth: u64) -> String {
        let mut result = String::new();
        let indent = TAB.repeat(depth as usize);
        result.push_str(&format!("{}TackyFunction: {}\n", indent, self.name_to_string()));
        result.push_str(&format!("{}{TAB}Instructions:\n", indent));
        for instruction in &self.instructions {
            result.push_str(&instruction.print_tacky_code(depth + 2));
        }
        result
    }
}

pub struct TackyProgram {
    pub function: TackyFunction,
    pop_context: Option<PoppedTokenContext>
}
impl TackyProgram {
    pub fn from_program(program: &ASTProgram) -> TackyProgram {
        return TackyProgram {
            pop_context: program.pop_context.clone(),
            function: TackyFunction::from_function(
                &program.function
            )
        };
    }
}
impl PrintableTacky for TackyProgram {
    fn print_tacky_code(&self, depth: u64) -> String {
        let mut result = String::new();
        let indent = TAB.repeat(depth as usize);
        result.push_str(&format!("{}TackyProgram:\n", indent));
        result.push_str(&*self.function.print_tacky_code(depth + 1));
        result
    }
}

pub fn tacky_gen_from_filepath(
    file_path: &str, verbose: bool
) -> Result<TackyProgram, ParseError> {
    let parse_result = parse_from_filepath(file_path, verbose);
    if parse_result.is_err() {
        return Err(parse_result.err().unwrap());
    }
    let program = parse_result?;
    let tacky_program = TackyProgram::from_program(&program);
    Ok(tacky_program)
}

