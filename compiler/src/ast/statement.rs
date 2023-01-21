use std::{fs::File, io::Write};

use crate::{
    codegen::{CodeGen, Compiler, SymbolTable},
    xml_printer::{print_closing, print_opening, print_symbol, print_tag, XmlPrinter},
};

use super::{expr::Expr, subroutine_call::SubroutineCall};

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

impl XmlPrinter for Vec<Statement> {
    fn print_xml(&self, file: &mut File) {
        print_opening(file, "statements");
        self.iter().for_each(|s| s.print_xml(file));
        print_closing(file, "statements");
    }
}

impl CodeGen for Statement {
    fn write_code(
        &self,
        out: &mut impl Write,
        compiler: &mut Compiler,
        symbol_table: &mut SymbolTable,
    ) {
        use Statement::*;

        match self {
            Let { name, access, expr } => todo!(),
            If {
                condition,
                if_true,
                if_false,
            } => todo!(),
            While {
                condition,
                statements,
            } => todo!(),
            // TODO: pop temp 0
            Do { call } => todo!(),
            Return { value } => todo!(),
        }
    }
}
