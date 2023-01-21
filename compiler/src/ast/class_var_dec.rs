use std::fs::File;

use crate::{xml_printer::{print_closing, print_opening, print_symbol, print_tag, XmlPrinter}, codegen::{CodeGen, SymbolScope}};

use super::{variable_scope::VariableScope, variable_type::VariableType};

#[derive(Debug, PartialEq, Eq)]
pub struct ClassVarDec {
    pub scope: VariableScope,
    pub typ: VariableType,
    pub vars: Vec<String>,
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

impl CodeGen for ClassVarDec {
    fn write_code(
        &self,
        _out: &mut impl std::io::Write,
        _compiler: &mut crate::codegen::Compiler,
        symbol_table: &mut crate::codegen::SymbolTable,
    ) {
        let scope = match self.scope {
            VariableScope::Static => SymbolScope::Static,
            VariableScope::Field => SymbolScope::Field,
            _ => panic!("Class-level variables must be fields or static variables."),
        };

        self.vars.iter().for_each(|v| {
            symbol_table.add_variable(v, &self.typ, scope);
        });
    }
}

impl CodeGen for Vec<ClassVarDec> {
    fn write_code(
        &self,
        out: &mut impl std::io::Write,
        compiler: &mut crate::codegen::Compiler,
        symbol_table: &mut crate::codegen::SymbolTable,
    ) {
        self.iter().for_each(|cvd| cvd.write_code(out, compiler, symbol_table));
    }
}
