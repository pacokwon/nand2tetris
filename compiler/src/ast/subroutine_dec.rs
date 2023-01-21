use std::fs::File;

use crate::{
    codegen::{pop, push, AsmSection, CodeGen},
    xml_printer::{print_closing, print_opening, print_symbol, print_tag, XmlPrinter},
};

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

impl CodeGen for SubroutineDec {
    fn write_code(
        &self,
        out: &mut impl std::io::Write,
        compiler: &mut crate::codegen::Compiler,
        symbol_table: &mut crate::codegen::SymbolTable,
    ) {
        use SubroutineKind::*;

        let Some(ref class) = compiler.current_class else {
            panic!("Subroutine must be declared inside a class.");
        };

        symbol_table.reset_local_table();
        writeln!(
            out,
            "function {}.{} {}",
            class.name,
            self.name,
            self.parameters.len()
        )
        .unwrap();

        match self.kind {
            Constructor => {
                // allocate memory for object.
                let size = std::cmp::min(1, class.fields_count);
                push(out, AsmSection::Constant, size);
                writeln!(out, "call Memory.alloc 1").unwrap();
                pop(out, AsmSection::Pointer, 0);
            }
            Function =>
            /* do nothing */
            {
                ()
            }
            Method => {
                // set `this` to the provided `this`
                push(out, AsmSection::Argument, 0);
                pop(out, AsmSection::Pointer, 0);
            }
        }

        self.body.write_code(out, compiler, symbol_table);
    }
}

impl CodeGen for Vec<SubroutineDec> {
    fn write_code(
        &self,
        out: &mut impl std::io::Write,
        compiler: &mut crate::codegen::Compiler,
        symbol_table: &mut crate::codegen::SymbolTable,
    ) {
        self.iter().for_each(|sd| sd.write_code(out, compiler, symbol_table));
    }
}
