use std::{fs::File, io::Write};

use crate::{
    codegen::{
        call_function, pop, push, push_constant, AsmSection, CodeGen, Compiler, SymbolTable,
    },
    xml_printer::{print_closing, print_opening, print_symbol, print_tag, XmlPrinter},
};

use super::{expr::Expr, subroutine_call::SubroutineCall};

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

impl CodeGen for ExprTerm {
    // push value to the top of the stack.
    fn write_code(
        &self,
        out: &mut impl Write,
        compiler: &mut Compiler,
        symbol_table: &mut SymbolTable,
    ) {
        use ExprTerm::*;

        match self {
            Integer(num) => writeln!(out, "push constant {num}").unwrap(),
            Str(ref s) => {
                let chars = s.chars().collect::<Vec<char>>();

                // Create string object first.
                push_constant(
                    out,
                    chars
                        .len()
                        .try_into()
                        .expect("String length is limited to 16 bits."),
                );
                call_function(out, "String.new", 1);

                chars.iter().for_each(|c| {
                    let codepoint = *c as u16;
                    push_constant(out, codepoint);
                    call_function(out, "String.appendChar", 2);
                });
            }
            True => {
                push_constant(out, 0);
                writeln!(out, "not").unwrap();
            }
            False => push_constant(out, 0),
            Null => push_constant(out, 0),
            This => writeln!(out, "push pointer 0").unwrap(),
            Variable(v) => {
                let entry = symbol_table
                    .resolve_variable(v)
                    .unwrap_or_else(|| panic!("Variable '{v}' not found in symbol table."));
                push(out, entry.scope.into(), entry.id);
            }
            Access(v, expr) => {
                expr.write_code(out, compiler, symbol_table);
                let entry = symbol_table
                    .resolve_variable(v)
                    .unwrap_or_else(|| panic!("Variable '{v}' not found in symbol table."));
                push(out, entry.scope.into(), entry.id);
                writeln!(out, "add").unwrap();
                pop(out, AsmSection::Pointer, 1);
                push(out, AsmSection::That, 0);
            }
            Call(sc) => sc.write_code(out, compiler, symbol_table),
            Group(expr) => expr.write_code(out, compiler, symbol_table),
            Unary(op, expr) => {
                expr.write_code(out, compiler, symbol_table);
                match op {
                    '-' => writeln!(out, "neg").unwrap(),
                    '~' => writeln!(out, "not").unwrap(),
                    _ => panic!("Operator '{op}' not supported as unary operator."),
                }
            }
        }
    }
}
