use std::fs::File;

use crate::xml_printer::{print_tag, XmlPrinter};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SubroutineKind {
    Constructor,
    Function,
    Method,
}

impl XmlPrinter for SubroutineKind {
    fn print_xml(&self, file: &mut File) {
        let kind = match self {
            SubroutineKind::Constructor => "constructor",
            SubroutineKind::Function => "function",
            SubroutineKind::Method => "method",
        };
        print_tag(file, "keyword", kind);
    }
}
