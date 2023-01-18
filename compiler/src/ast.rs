#[derive(Debug)]
pub enum VariableScope {
    Static,
    Field,
    Local,
}

#[derive(Debug)]
pub enum VariableType {
    Int,
    Char,
    Boolean,
    Void,
    Other(String),
}

#[derive(Debug)]
pub enum SubroutineKind {
    Constructor,
    Function,
    Method,
}

#[derive(Debug)]
pub struct ClassVarDec {
    pub scope: VariableScope,
    pub typ: VariableType,
    pub vars: Vec<String>,
}

#[derive(Debug)]
pub struct VarDec {
    pub typ: VariableType,
    pub vars: Vec<String>,
}

#[derive(Debug)]
pub struct SubroutineBody {
    pub locals: Vec<VarDec>,
    pub statements: Vec<Statement>,
}

#[derive(Debug)]
pub struct SubroutineDec {
    pub kind: SubroutineKind,
    pub return_type: VariableType,
    pub name: String,
    pub parameters: Vec<(VariableType, String)>,
    pub body: SubroutineBody,
}

#[derive(Debug)]
pub struct Class {
    pub name: String,
    pub variables: Vec<ClassVarDec>,
    pub subroutines: Vec<SubroutineDec>,
}

#[derive(Debug)]
pub enum Statement {
    Let{name: String, access: Option<Expr>, expr: Expr},
    If{condition: Expr, if_true: Vec<Statement>, if_false: Option<Vec<Statement>>},
    While{condition: Expr, statements: Vec<Statement>},
    Do{call: SubroutineCall},
    Return{value: Option<Expr>},
}

#[derive(Debug)]
pub struct Expr {
    pub lhs: ExprTerm,
    pub rhs: Vec<(char, ExprTerm)>,
}

#[derive(Debug)]
pub enum SubroutineCall {
    Function(String, Vec<Expr>),
    Method(String, String, Vec<Expr>),
}

#[derive(Debug)]
pub enum ExprTerm {
    Integer(u16),
    Str(String),
    True,
    False,
    Null,
    This,
    Variable(String),
    Access(String, Box<Expr>),
    Call(SubroutineCall),
    Group(Box<Expr>),
    Unary(char, Box<ExprTerm>),
}
