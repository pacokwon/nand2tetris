use std::{fs::File, io::Write};

use crate::{
    codegen::{CodeGen, Compiler, SymbolTable},
    xml_printer::{print_closing, print_opening, print_symbol, XmlPrinter},
};

use super::expr_term::ExprTerm;

#[derive(Debug, PartialEq, Eq)]
pub struct Expr {
    pub lhs: ExprTerm,
    pub rhs: Vec<(char, ExprTerm)>,
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

impl CodeGen for Expr {
    fn write_code(
        &self,
        out: &mut impl Write,
        compiler: &mut Compiler,
        symbol_table: &mut SymbolTable,
    ) {
        self.lhs.write_code(out, compiler, symbol_table);

        self.rhs.iter().for_each(|(op, expr)| {
            expr.write_code(out, compiler, symbol_table);
            let inst = match op {
                '+' => "add",
                '-' => "sub",
                '*' => "call Math.multiply 2",
                '/' => "call Math.divide 2",
                '&' => "and",
                '|' => "or",
                '<' => "lt",
                '>' => "gt",
                '=' => "eq",
                _ => panic!("Operator '{op}' not supported as unary operator."),
            };
            writeln!(out, "{}", inst).unwrap();
        });
    }
}

impl CodeGen for Vec<Expr> {
    fn write_code(
        &self,
        out: &mut impl Write,
        compiler: &mut Compiler,
        symbol_table: &mut SymbolTable,
    ) {
        self.iter()
            .for_each(|expr| expr.write_code(out, compiler, symbol_table));
    }
}
