use std::fs::File;

use crate::{xml_printer::{print_closing, print_opening, print_symbol, print_tag, XmlPrinter}, codegen::{CodeGen, SymbolScope}};

use super::variable_type::VariableType;

#[derive(Debug, PartialEq, Eq)]
pub struct VarDec {
    pub typ: VariableType,
    pub vars: Vec<String>,
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

impl CodeGen for VarDec {
    fn write_code(&self, _out: &mut impl std::io::Write, _compiler: &mut crate::codegen::Compiler, symbol_table: &mut crate::codegen::SymbolTable) {
        // local variable declaration.
        // no vm instruction to emit.
        // we update the symbol table here.

        // we have default types char, int, boolean
        // and we have reference types.
        self.vars.iter().for_each(|v| {
            symbol_table.add_variable(v, &self.typ, SymbolScope::Local);
        });
    }
}

impl CodeGen for Vec<VarDec> {
    fn write_code(
        &self,
        out: &mut impl std::io::Write,
        compiler: &mut crate::codegen::Compiler,
        symbol_table: &mut crate::codegen::SymbolTable,
    ) {
        self.iter().for_each(|v| v.write_code(out, compiler, symbol_table));
    }
}
