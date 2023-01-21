use std::{fs::File, io::Write};

use crate::{
    codegen::{pop, push, AsmSection, CodeGen, Compiler, SymbolTable},
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
            Let { name, access, expr } => {
                expr.write_code(out, compiler, symbol_table);

                if let Some(access_expr) = access {
                    access_expr.write_code(out, compiler, symbol_table);

                    let entry = symbol_table
                        .resolve_variable(name)
                        .unwrap_or_else(|| panic!("Variable '{name}' not found in symbol table."));
                    push(out, entry.scope.into(), entry.id);
                    writeln!(out, "add").unwrap();
                    pop(out, AsmSection::Pointer, 1);
                    pop(out, AsmSection::That, 0);
                } else {
                    let entry = symbol_table
                        .resolve_variable(name)
                        .unwrap_or_else(|| panic!("Variable '{name}' not found in symbol table."));
                    pop(out, entry.scope.into(), entry.id);
                }
            }
            If {
                condition,
                if_true,
                if_false,
            } => {
                let label = compiler.get_new_branch_counter();
                let false_branch_exists = if_false.is_some();

                condition.write_code(out, compiler, symbol_table);
                write!(
                    out,
                    "\
                        if-goto IF_TRUE{label}\n\
                        goto IF_FALSE{label}\n\
                        label IF_TRUE{label}\n\
                    "
                )
                .unwrap();
                if_true.write_code(out, compiler, symbol_table);

                if false_branch_exists {
                    writeln!(out, "goto IF_END{label}").unwrap();
                }

                writeln!(out, "label IF_FALSE{label}").unwrap();

                if false_branch_exists {
                    if_false
                        .as_ref()
                        .unwrap()
                        .write_code(out, compiler, symbol_table);
                    writeln!(out, "label IF_END{label}").unwrap();
                }
            }
            While {
                condition,
                statements,
            } => {
                let label = compiler.get_new_branch_counter();
                writeln!(out, "label WHILE_EXP{label}").unwrap();
                condition.write_code(out, compiler, symbol_table);
                // negate the condition
                writeln!(out, "not").unwrap();

                // if the negated condition is true, the while loop is over.
                writeln!(out, "if-goto WHILE_END{label}").unwrap();

                // otherwise, run the body.
                statements.write_code(out, compiler, symbol_table);

                writeln!(out, "goto WHILE_EXP{label}").unwrap();
                writeln!(out, "label WHILE_END{label}").unwrap();
            }
            Do { call } => {
                call.write_code(out, compiler, symbol_table);
                // move the return value to a temporary variable
                pop(out, AsmSection::Temp, 0);
            }
            Return { value } => {
                if let Some(expr) = value {
                    expr.write_code(out, compiler, symbol_table);
                } else {
                    push(out, AsmSection::Constant, 0);
                }
                writeln!(out, "return").unwrap();
            }
        }
    }
}

impl CodeGen for Vec<Statement> {
    fn write_code(
        &self,
        out: &mut impl Write,
        compiler: &mut Compiler,
        symbol_table: &mut SymbolTable,
    ) {
        self.iter()
            .for_each(|s| s.write_code(out, compiler, symbol_table));
    }
}
