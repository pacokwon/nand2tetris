use std::{fs::File, io::Write};

use crate::{
    codegen::{push, push_this, CodeGen, Compiler, SymbolTable},
    xml_printer::{print_symbol, print_tag, XmlPrinter},
};

use super::expr::Expr;

#[derive(Debug, PartialEq, Eq)]
pub enum SubroutineCall {
    Function(String, Vec<Expr>),
    Method(String, String, Vec<Expr>),
}

impl XmlPrinter for SubroutineCall {
    fn print_xml(&self, file: &mut File) {
        match self {
            SubroutineCall::Function(name, args) => {
                print_tag(file, "identifier", name);
                print_symbol(file, "(");
                args.print_xml(file);
                print_symbol(file, ")");
            }
            SubroutineCall::Method(module, name, args) => {
                print_tag(file, "identifier", module);
                print_symbol(file, ".");
                print_tag(file, "identifier", name);
                print_symbol(file, "(");
                args.print_xml(file);
                print_symbol(file, ")");
            }
        }
    }
}

impl CodeGen for SubroutineCall {
    fn write_code(
        &self,
        out: &mut impl Write,
        compiler: &mut Compiler,
        symbol_table: &mut SymbolTable,
    ) {
        use SubroutineCall::*;

        match self {
            Function(func_name, args) => {
                // if there is no module, then this is a method call.
                if !compiler.is_inside_method {
                    panic!("Methods can only be called inside methods or constructors.");
                }

                // push `this` first.
                push_this(out);

                // push arguments to stack.
                args.write_code(out, compiler, symbol_table);

                let Some(ref class_name) = compiler.current_class else {
                    panic!("Method call is supposed to be inside a class.");
                };
                // call function.
                let full_name = format!("{class_name}.{func_name}");
                // we add 1 to arguments length since `this` is being injected.
                writeln!(out, "call {} {}", full_name, args.len() + 1).unwrap();
            }
            Method(namespace, method, args) => {
                let Some(ref class_name) = compiler.current_class else {
                    panic!("Method/Function call is supposed to be inside a class.");
                };
                // check if module is class / variable
                if class_name == namespace {
                    // module is class => this is a function call.

                    // push arguments to stack
                    args.write_code(out, compiler, symbol_table);

                    // call function
                    writeln!(out, "call {namespace}.{method} {}", args.len()).unwrap();
                } else if let Some(entry) = symbol_table.resolve_variable(namespace) {
                    // module is variable => this is a method call

                    // we should push the variable's `this`
                    // ex> compilation of bat.dispose() should result in
                    // pushing bat's `this` to the stack first.
                    push(out, entry.scope.into(), entry.id);
                    // push rest of arguments.
                    args.write_code(out, compiler, symbol_table);

                    writeln!(out, "call {namespace}.{method} {}", args.len() + 1).unwrap();
                } else {
                    panic!("{namespace} must be a class name or a variable.");
                }
            }
        }
    }
}
