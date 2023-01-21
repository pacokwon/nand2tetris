use std::{collections::HashMap, io::Write};

use crate::ast::class::Class;

#[derive(Debug)]
pub struct Compiler {
    pub class_counter: u16,
    pub local_counter: u16,
    pub is_inside_method: bool,
    pub current_class: Option<String>,
}

impl Compiler {
    pub fn new() -> Self {
        let class_counter = 0;
        let local_counter = 0;
        let is_inside_method = false;
        let current_class = None;

        Compiler {
            class_counter,
            local_counter,
            is_inside_method,
            current_class,
        }
    }

    pub fn compile(&mut self, ast: &Class) {
        let mut symbol_table = SymbolTable::new();
        ast.write_code(&mut std::io::stdout(), self, &mut symbol_table);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SymbolScope {
    Static,
    Field,
    Local,
    Argument,
}

impl Into<AsmSection> for SymbolScope {
    fn into(self) -> AsmSection {
        match self {
            SymbolScope::Static => AsmSection::Static,
            SymbolScope::Field => AsmSection::This,
            SymbolScope::Local => AsmSection::Local,
            SymbolScope::Argument => AsmSection::Argument,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct SymbolEntry {
    pub id: u16,
    pub scope: SymbolScope,
}

pub struct SymbolTable {
    pub class_symbols: HashMap<String, SymbolEntry>,
    pub local_symbols: HashMap<String, SymbolEntry>,
}

impl SymbolTable {
    pub fn new() -> Self {
        let class_symbols = HashMap::new();
        let local_symbols = HashMap::new();

        SymbolTable {
            class_symbols,
            local_symbols,
        }
    }

    pub fn resolve_variable(&self, name: &str) -> Option<SymbolEntry> {
        if let Some(entry) = self.local_symbols.get(name) {
            Some(*entry)
        } else if let Some(entry) = self.class_symbols.get(name) {
            Some(*entry)
        } else {
            None
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AsmSection {
    Constant,
    Static,
    Local,
    Argument,
    This,
    That,
    Pointer,
}

pub trait CodeGen {
    fn write_code(&self, out: &mut impl Write, compiler: &mut Compiler, symbol_table: &mut SymbolTable);
}

pub fn push_constant(out: &mut impl Write, val: u16) {
    writeln!(out, "push constant {val}").unwrap();
}

// TODO: is this sufficient?
pub fn push_this(out: &mut impl Write) {
    writeln!(out, "push pointer 0").unwrap();
}

pub fn call_function(out: &mut impl Write, func: &str, args: u16) {
    writeln!(out, "call {func} {args}").unwrap();
}

pub fn push(out: &mut impl Write, section: AsmSection, index: u16) {
    use AsmSection::*;

    match section {
        Constant => writeln!(out, "push constant {index}").unwrap(),
        Static => writeln!(out, "push static {index}").unwrap(),
        Local => writeln!(out, "push local {index}").unwrap(),
        Argument => writeln!(out, "push argument {index}").unwrap(),
        This => writeln!(out, "push this {index}").unwrap(),
        That => writeln!(out, "push that {index}").unwrap(),
        Pointer => writeln!(out, "push pointer {index}").unwrap(),
    }
}

pub fn pop(out: &mut impl Write, section: AsmSection, index: u16) {
    use AsmSection::*;

    match section {
        Constant => writeln!(out, "pop constant {index}").unwrap(),
        Static => writeln!(out, "pop static {index}").unwrap(),
        Local => writeln!(out, "pop local {index}").unwrap(),
        Argument => writeln!(out, "pop argument {index}").unwrap(),
        This => writeln!(out, "pop this {index}").unwrap(),
        That => writeln!(out, "pop that {index}").unwrap(),
        Pointer => writeln!(out, "pop pointer {index}").unwrap(),
    }
}
