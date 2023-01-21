use std::fs::File;

use crate::xml_printer::{print_tag, XmlPrinter};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VariableType {
    Int,
    Char,
    Boolean,
    Void,
    Other(String),
}

impl XmlPrinter for VariableType {
    fn print_xml(&self, file: &mut File) {
        match self {
            VariableType::Int => print_tag(file, "keyword", "int"),
            VariableType::Char => print_tag(file, "keyword", "char"),
            VariableType::Boolean => print_tag(file, "keyword", "boolean"),
            VariableType::Void => print_tag(file, "keyword", "void"),
            VariableType::Other(ref s) => print_tag(file, "identifier", s),
        };
    }
}
