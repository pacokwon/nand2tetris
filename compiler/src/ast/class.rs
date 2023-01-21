use std::{fs::File, io::Write};

use crate::{
    codegen::{CodeGen, Compiler, SymbolTable},
    xml_printer::{print_closing, print_opening, print_symbol, print_tag, XmlPrinter},
};

use super::{
    class_var_dec::ClassVarDec, subroutine_dec::SubroutineDec, variable_scope::VariableScope,
};

#[derive(Debug, PartialEq, Eq)]
pub struct Class {
    pub name: String,
    pub variables: Vec<ClassVarDec>,
    pub subroutines: Vec<SubroutineDec>,
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

impl CodeGen for Class {
    fn write_code(
        &self,
        out: &mut impl Write,
        compiler: &mut Compiler,
        symbol_table: &mut SymbolTable,
    ) {
        let size = self
            .variables
            .iter()
            // only count fields, not static variables
            .filter(|cv| {
                if let VariableScope::Field = cv.scope {
                    true
                } else {
                    false
                }
            })
            .map(|cv| cv.vars.len())
            .sum::<usize>() as u16;
        compiler.set_current_class(&self.name, size);
        self.variables.write_code(out, compiler, symbol_table);
        self.subroutines.write_code(out, compiler, symbol_table);
    }
}
