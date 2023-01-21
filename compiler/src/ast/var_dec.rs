use std::fs::File;

use crate::xml_printer::{print_closing, print_opening, print_symbol, print_tag, XmlPrinter};

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
