use std::fs::File;

use crate::xml_printer::{print_tag, XmlPrinter};

#[derive(Debug, PartialEq, Eq)]
pub enum VariableScope {
    Static,
    Field,
    Local,
}

impl XmlPrinter for VariableScope {
    fn print_xml(&self, file: &mut File) {
        let scope = match self {
            VariableScope::Static => "static",
            VariableScope::Field => "field",
            VariableScope::Local => "local",
        };
        print_tag(file, "keyword", scope);
    }
}
