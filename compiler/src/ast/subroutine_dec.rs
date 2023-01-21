use std::fs::File;

use crate::xml_printer::{print_closing, print_opening, print_symbol, print_tag, XmlPrinter};

use super::{
    subroutine_body::SubroutineBody, subroutine_kind::SubroutineKind, variable_type::VariableType,
};

#[derive(Debug, PartialEq, Eq)]
pub struct SubroutineDec {
    pub kind: SubroutineKind,
    pub return_type: VariableType,
    pub name: String,
    pub parameters: Vec<(VariableType, String)>,
    pub body: SubroutineBody,
}

impl XmlPrinter for SubroutineDec {
    fn print_xml(&self, file: &mut File) {
        print_opening(file, "subroutineDec");

        self.kind.print_xml(file);

        self.return_type.print_xml(file);
        print_tag(file, "identifier", &self.name);

        print_symbol(file, "(");

        print_opening(file, "parameterList");

        if !self.parameters.is_empty() {
            self.parameters[0].0.print_xml(file);
            print_tag(file, "identifier", &self.parameters[0].1);

            self.parameters.iter().skip(1).for_each(|(typ, name)| {
                print_symbol(file, ",");
                typ.print_xml(file);
                print_tag(file, "identifier", name);
            });
        }
        print_closing(file, "parameterList");

        print_symbol(file, ")");

        self.body.print_xml(file);
        print_closing(file, "subroutineDec");
    }
}
