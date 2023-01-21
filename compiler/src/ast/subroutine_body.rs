use std::fs::File;

use crate::xml_printer::{print_closing, print_opening, print_symbol, XmlPrinter};

use super::{statement::Statement, var_dec::VarDec};

#[derive(Debug, PartialEq, Eq)]
pub struct SubroutineBody {
    pub locals: Vec<VarDec>,
    pub statements: Vec<Statement>,
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
