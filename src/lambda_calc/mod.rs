use std::fmt;

#[derive(Debug, Clone)]
pub struct LambdaVariable {
    id: u64,
    name: String,
}
impl LambdaVariable {
    pub fn new(id: u64) -> LambdaVariable {
        LambdaVariable { id, name: "".to_string() }
    }
}
impl Eq for LambdaVariable {}
impl PartialEq for LambdaVariable {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

pub enum LambdaExpression {
    Var(LambdaVariable),
    Application(Box<LambdaExpression>, Box<LambdaExpression>),
    Function(LambdaVariable, Box<LambdaExpression>),
}
impl LambdaExpression {
    pub fn to_string(&self) -> String {
        match self {
            LambdaExpression::Var(v) => v.name.clone(),
            LambdaExpression::Application(e1, e2) => {
                format!("({} {})", e1.to_string(), e2.to_string())
            },
            LambdaExpression::Function(param, body) => {
                format!("(λ{}. {})", param.name, body.to_string())
            },
        }
    }
    pub fn replace(
        &self, var: &LambdaVariable, expr: &LambdaExpression
    ) -> LambdaExpression {
        match self {
            LambdaExpression::Var(v) => {
                if v == var {
                    expr.clone()
                } else {
                    self.clone()
                }
            },
            LambdaExpression::Application(e1, e2) => {
                LambdaExpression::Application(
                    Box::new(e1.replace(var, &expr.clone())),
                    Box::new(e2.replace(var, &expr.clone()))
                )
            },
            LambdaExpression::Function(param, body) => {
                if param == var {
                    self.clone()
                } else {
                    LambdaExpression::Function(
                        param.clone(),
                        Box::new(body.replace(var, expr))
                    )
                }
            },
        }
    }
}
impl fmt::Display for LambdaExpression {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_string())
    }
}
impl Clone for LambdaExpression {
    fn clone(&self) -> Self {
        match self {
            LambdaExpression::Var(v) => LambdaExpression::Var(v.clone()),
            LambdaExpression::Application(e1, e2) => {
                LambdaExpression::Application(
                    Box::from(*e1.clone()), Box::from(*e2.clone())
                )
            },
            LambdaExpression::Function(param, body) => {
                LambdaExpression::Function(
                    param.clone(), Box::new(*body.clone())
                )
            },
        }
    }
}