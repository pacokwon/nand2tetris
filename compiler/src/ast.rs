use std::fs::File;
use std::io::Write;

use crate::xml_printer::{print_closing, print_opening, print_symbol, print_tag, XmlPrinter};

#[derive(Debug, PartialEq, Eq)]
pub enum VariableScope {
    Static,
    Field,
    Local,
}

#[derive(Debug, PartialEq, Eq)]
pub enum VariableType {
    Int,
    Char,
    Boolean,
    Void,
    Other(String),
}

#[derive(Debug, PartialEq, Eq)]
pub enum SubroutineKind {
    Constructor,
    Function,
    Method,
}

#[derive(Debug, PartialEq, Eq)]
pub struct ClassVarDec {
    pub scope: VariableScope,
    pub typ: VariableType,
    pub vars: Vec<String>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct VarDec {
    pub typ: VariableType,
    pub vars: Vec<String>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct SubroutineBody {
    pub locals: Vec<VarDec>,
    pub statements: Vec<Statement>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct SubroutineDec {
    pub kind: SubroutineKind,
    pub return_type: VariableType,
    pub name: String,
    pub parameters: Vec<(VariableType, String)>,
    pub body: SubroutineBody,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Class {
    pub name: String,
    pub variables: Vec<ClassVarDec>,
    pub subroutines: Vec<SubroutineDec>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Statement {
    Let {
        name: String,
        access: Option<Expr>,
        expr: Expr,
    },
    If {
        condition: Expr,
        if_true: Vec<Statement>,
        if_false: Option<Vec<Statement>>,
    },
    While {
        condition: Expr,
        statements: Vec<Statement>,
    },
    Do {
        call: SubroutineCall,
    },
    Return {
        value: Option<Expr>,
    },
}

#[derive(Debug, PartialEq, Eq)]
pub struct Expr {
    pub lhs: ExprTerm,
    pub rhs: Vec<(char, ExprTerm)>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum SubroutineCall {
    Function(String, Vec<Expr>),
    Method(String, String, Vec<Expr>),
}

#[derive(Debug, PartialEq, Eq)]
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

impl XmlPrinter for ExprTerm {
    fn print_xml(&self, file: &mut File) {
        use ExprTerm::*;

        print_opening(file, "term");

        match self {
            Integer(num) => writeln!(file, "<integerConstant> {} </integerConstant>", num).unwrap(),
            Str(s) => print_tag(file, "stringConstant", s),
            True => print_tag(file, "keyword", "true"),
            False => print_tag(file, "keyword", "false"),
            Null => print_tag(file, "keyword", "null"),
            This => print_tag(file, "keyword", "this"),
            Variable(v) => print_tag(file, "identifier", v),
            Access(v, expr) => {
                print_tag(file, "identifier", v);
                print_symbol(file, "[");
                expr.print_xml(file);
                print_symbol(file, "]");
            }
            Call(sc) => sc.print_xml(file),
            Group(expr) => {
                print_symbol(file, "(");
                expr.print_xml(file);
                print_symbol(file, ")");
            }
            Unary(op, term) => {
                op.print_xml(file);
                term.print_xml(file);
            }
        }

        print_closing(file, "term");
    }
}

impl XmlPrinter for ClassVarDec {
    fn print_xml(&self, file: &mut File) {
        print_opening(file, "classVarDec");

        self.scope.print_xml(file);
        self.typ.print_xml(file);
        print_tag(file, "identifier", &self.vars[0]);
        self.vars.iter().skip(1).for_each(|v| {
            print_symbol(file, ",");
            print_tag(file, "identifier", v);
        });
        print_symbol(file, ";");

        print_closing(file, "classVarDec");
    }
}

impl XmlPrinter for Statement {
    fn print_xml(&self, file: &mut File) {
        match self {
            Statement::Let {
                ref name,
                access,
                expr,
            } => {
                print_opening(file, "letStatement");
                print_tag(file, "keyword", "let");
                print_tag(file, "identifier", name);
                if let Some(ref access_expr) = access {
                    print_symbol(file, "[");
                    access_expr.print_xml(file);
                    print_symbol(file, "]");
                }
                print_symbol(file, "=");
                expr.print_xml(file);
                print_symbol(file, ";");
                print_closing(file, "letStatement");
            }
            Statement::If {
                condition,
                if_true,
                if_false,
            } => {
                print_opening(file, "ifStatement");
                print_tag(file, "keyword", "if");
                print_symbol(file, "(");
                condition.print_xml(file);
                print_symbol(file, ")");
                print_symbol(file, "{");
                if_true.print_xml(file);
                print_symbol(file, "}");

                if let Some(ref if_false_stmts) = if_false {
                    print_tag(file, "keyword", "else");
                    print_symbol(file, "{");
                    if_false_stmts.print_xml(file);
                    print_symbol(file, "}");
                }

                print_closing(file, "ifStatement");
            }
            Statement::While {
                condition,
                statements,
            } => {
                print_opening(file, "whileStatement");
                print_tag(file, "keyword", "while");
                print_symbol(file, "(");
                condition.print_xml(file);
                print_symbol(file, ")");
                print_symbol(file, "{");
                statements.print_xml(file);
                print_symbol(file, "}");
                print_closing(file, "whileStatement");
            }
            Statement::Do { call } => {
                print_opening(file, "doStatement");
                print_tag(file, "keyword", "do");
                call.print_xml(file);
                print_symbol(file, ";");
                print_closing(file, "doStatement");
            }
            Statement::Return { value } => {
                print_opening(file, "returnStatement");
                print_tag(file, "keyword", "return");
                if let Some(expr) = value {
                    expr.print_xml(file);
                }
                print_symbol(file, ";");
                print_closing(file, "returnStatement");
            }
        }
    }
}

impl XmlPrinter for Class {
    fn print_xml(&self, file: &mut File) {
        print_opening(file, "class");

        print_tag(file, "keyword", "class");
        print_tag(file, "identifier", &self.name);
        print_symbol(file, "{");
        self.variables.iter().for_each(|v| v.print_xml(file));
        self.subroutines.iter().for_each(|s| s.print_xml(file));
        print_symbol(file, "}");

        print_closing(file, "class");
    }
}

impl XmlPrinter for Expr {
    fn print_xml(&self, file: &mut File) {
        print_opening(file, "expression");

        self.lhs.print_xml(file);

        self.rhs.iter().for_each(|(op, expr)| {
            op.print_xml(file);
            expr.print_xml(file);
        });

        print_closing(file, "expression");
    }
}

impl XmlPrinter for SubroutineBody {
    fn print_xml(&self, file: &mut File) {
        print_opening(file, "subroutineBody");
        print_symbol(file, "{");

        self.locals.iter().for_each(|l| l.print_xml(file));
        self.statements.print_xml(file);

        print_symbol(file, "}");
        print_closing(file, "subroutineBody");
    }
}

impl XmlPrinter for SubroutineCall {
    fn print_xml(&self, file: &mut File) {
        match self {
            SubroutineCall::Function(name, args) => {
                print_tag(file, "identifier", name);
                print_symbol(file, "(");
                args.print_xml(file);
                print_symbol(file, ")");
            }
            SubroutineCall::Method(module, name, args) => {
                print_tag(file, "identifier", module);
                print_symbol(file, ".");
                print_tag(file, "identifier", name);
                print_symbol(file, "(");
                args.print_xml(file);
                print_symbol(file, ")");
            }
        }
    }
}

impl XmlPrinter for SubroutineDec {
    fn print_xml(&self, file: &mut File) {
        print_opening(file, "subroutineDec");

        self.kind.print_xml(file);

        self.return_type.print_xml(file);
        print_tag(file, "identifier", &self.name);

        print_symbol(file, "(");

        print_opening(file, "parameterList");

        if !self.parameters.is_empty() {
            self.parameters[0].0.print_xml(file);
            print_tag(file, "identifier", &self.parameters[0].1);

            self.parameters.iter().skip(1).for_each(|(typ, name)| {
                print_symbol(file, ",");
                typ.print_xml(file);
                print_tag(file, "identifier", name);
            });
        }
        print_closing(file, "parameterList");

        print_symbol(file, ")");

        self.body.print_xml(file);
        print_closing(file, "subroutineDec");
    }
}

impl XmlPrinter for SubroutineKind {
    fn print_xml(&self, file: &mut File) {
        let kind = match self {
            SubroutineKind::Constructor => "constructor",
            SubroutineKind::Function => "function",
            SubroutineKind::Method => "method",
        };
        print_tag(file, "keyword", kind);
    }
}

impl XmlPrinter for VarDec {
    fn print_xml(&self, file: &mut File) {
        print_opening(file, "varDec");

        print_tag(file, "keyword", "var");
        self.typ.print_xml(file);

        print_tag(file, "identifier", &self.vars[0]);
        self.vars.iter().skip(1).for_each(|v| {
            print_symbol(file, ",");
            print_tag(file, "identifier", v);
        });

        print_symbol(file, ";");
        print_closing(file, "varDec");
    }
}

impl XmlPrinter for VariableType {
    fn print_xml(&self, file: &mut File) {
        match self {
            VariableType::Int => print_tag(file, "keyword", "int"),
            VariableType::Char => print_tag(file, "keyword", "char"),
            VariableType::Boolean => print_tag(file, "keyword", "boolean"),
            VariableType::Void => print_tag(file, "keyword", "void"),
            VariableType::Other(ref s) => print_tag(file, "identifier", s),
        };
    }
}

impl XmlPrinter for VariableScope {
    fn print_xml(&self, file: &mut File) {
        let scope = match self {
            VariableScope::Static => "static",
            VariableScope::Field => "field",
            VariableScope::Local => "local",
        };
        print_tag(file, "keyword", scope);
    }
}

impl XmlPrinter for Vec<Statement> {
    fn print_xml(&self, file: &mut File) {
        print_opening(file, "statements");
        self.iter().for_each(|s| s.print_xml(file));
        print_closing(file, "statements");
    }
}

impl XmlPrinter for Vec<Expr> {
    fn print_xml(&self, file: &mut File) {
        print_opening(file, "expressionList");
        if !self.is_empty() {
            self[0].print_xml(file);
            self.iter().skip(1).for_each(|e| {
                print_symbol(file, ",");
                e.print_xml(file)
            });
        }

        print_closing(file, "expressionList");
    }
}
