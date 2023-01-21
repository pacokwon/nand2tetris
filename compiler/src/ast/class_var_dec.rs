use std::fs::File;

use crate::xml_printer::{print_closing, print_opening, print_symbol, print_tag, XmlPrinter};

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
