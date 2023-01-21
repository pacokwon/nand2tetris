use std::{collections::HashMap, io::Write};

use crate::ast::{class::Class, variable_type::VariableType, subroutine_kind::SubroutineKind};

#[derive(Debug)]
pub struct ClassInfo {
    pub name: String,
    pub fields_count: u16,
}

#[derive(Debug)]
pub struct Compiler {
    pub current_subroutine_kind: Option<SubroutineKind>,
    pub current_class: Option<ClassInfo>,
    branch_counter: u16,
}

impl Compiler {
    pub fn new() -> Self {
        let current_subroutine_kind = None;
        let current_class = None;
        let branch_counter = 0;

        Compiler {
            current_subroutine_kind,
            current_class,
            branch_counter,
        }
    }

    pub fn compile(&mut self, ast: &Class, out: &mut impl Write) {
        let mut symbol_table = SymbolTable::new();
        ast.write_code(out, self, &mut symbol_table);
    }

    pub fn get_new_branch_counter(&mut self) -> u16 {
        self.branch_counter += 1;
        self.branch_counter
    }

    pub fn set_current_class(&mut self, name: &str, fields_count: u16) {
        self.current_class = Some(ClassInfo {
            name: name.to_string(),
            fields_count,
        });
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

#[derive(Debug)]
pub struct SymbolEntry {
    pub id: u16,
    pub typ: VariableType,
    pub scope: SymbolScope,
}

pub struct SymbolTable {
    pub class_symbols: HashMap<String, SymbolEntry>,
    pub local_symbols: HashMap<String, SymbolEntry>,
    static_counter: i32,
    field_counter: i32,
    local_counter: i32,
    args_counter: i32,
}

impl SymbolTable {
    pub fn new() -> Self {
        let class_symbols = HashMap::new();
        let local_symbols = HashMap::new();
        let static_counter = -1;
        let field_counter = -1;
        let local_counter = -1;
        let args_counter = -1;

        SymbolTable {
            class_symbols,
            local_symbols,
            static_counter,
            field_counter,
            local_counter,
            args_counter,
        }
    }

    pub fn resolve_variable(&self, name: &str) -> Option<&SymbolEntry> {
        if let Some(entry) = self.local_symbols.get(name) {
            Some(entry)
        } else if let Some(entry) = self.class_symbols.get(name) {
            Some(entry)
        } else {
            None
        }
    }

    pub fn add_variable(&mut self, name: &str, typ: &VariableType, scope: SymbolScope) {
        let typ = typ.clone();
        let id = match scope {
            SymbolScope::Static => {
                self.static_counter += 1;
                self.static_counter
            },
            SymbolScope::Field => {
                self.field_counter += 1;
                self.field_counter
            },
            SymbolScope::Local => {
                self.local_counter += 1;
                self.local_counter
            },
            SymbolScope::Argument => {
                self.args_counter += 1;
                self.args_counter
            },
        } as u16;

        match scope {
            SymbolScope::Static | SymbolScope::Field => {
                if self.class_symbols.contains_key(name) {
                    panic!("Duplicate variable '{name}' in class scope.");
                }

                self.class_symbols
                    .insert(name.to_string(), SymbolEntry { id, typ, scope });
            }
            SymbolScope::Local | SymbolScope::Argument => {
                if self.local_symbols.contains_key(name) {
                    panic!("Duplicate variable '{name}' in local scope.");
                }

                self.local_symbols
                    .insert(name.to_string(), SymbolEntry { id, typ, scope });
            }
        }
    }

    pub fn reset_local_table(&mut self, subroutine_kind: SubroutineKind) {
        use SubroutineKind::*;

        match subroutine_kind {
            Constructor | Function => {
                self.args_counter = -1;
            },
            Method => {
                self.args_counter = 0;
            },
        }
        self.local_counter = -1;
        self.local_symbols = HashMap::new();
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
    Temp,
}

pub trait CodeGen {
    fn write_code(
        &self,
        out: &mut impl Write,
        compiler: &mut Compiler,
        symbol_table: &mut SymbolTable,
    );
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
        Temp => writeln!(out, "push temp {index}").unwrap(),
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
        Temp => writeln!(out, "pop temp {index}").unwrap(),
    }
}
