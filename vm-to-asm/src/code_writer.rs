use std::{fs::File, io::Write};

use crate::command::CommandType;

pub struct CodeWriter {
    output_file: Option<File>,
    module: Option<String>,
    jump_counter: u16,
    return_counter: u16,
}

impl CodeWriter {
    pub fn new() -> Self {
        CodeWriter {
            output_file: None,
            module: None,
            jump_counter: 0,
            return_counter: 0,
        }
    }

    pub fn set_output_filename(&mut self, name: &str) {
        let file = File::create(name).expect("set_filename: file not found");
        self.output_file = Some(file);
        self.jump_counter = 0;
    }

    pub fn set_module_name(&mut self, name: &str) {
        self.module = Some(name.to_string());
    }

    pub fn close(&mut self) {
        self.output_file = None;
    }

    pub fn write_init(&mut self) {
        let out_file = self
            .output_file
            .as_mut()
            .expect("Target file not set. Call set_filename() before writing commands.");

        write!(
            out_file,
            "\
                @256\n\
                D=A\n\
                @SP\n\
                M=D\n\
            "
        )
        .unwrap();

        let function_name = "Sys.init";
        let args_count = 0;

        self.return_counter += 1;
        let return_address_label = Self::get_return_symbol(function_name, self.return_counter);

        write!(
            out_file,
            "\
                @{return_address_label}\n\
                D=A\n\
                {}",
            Self::get_push_code()
        )
        .unwrap();

        Self::push_symbol(out_file, "LCL");
        Self::push_symbol(out_file, "ARG");
        Self::push_symbol(out_file, "THIS");
        Self::push_symbol(out_file, "THAT");

        // ARG = SP-n-5,
        // LCL = SP
        write!(
            out_file,
            "\
                @SP\n\
                D=M\n\
                @LCL\n\
                M=D\n\
                @5\n\
                D=D-A\n\
                @{args_count}\n\
                D=D-A\n\
                @ARG\n\
                M=D\n\
                @{function_name}\n\
                0;JMP\n\
                ({return_address_label})\n\
            "
        )
        .unwrap();
    }

    pub fn write_arithemtic(&mut self, command: &str) {
        let out_file = self
            .output_file
            .as_mut()
            .expect("Target file not set. Call set_filename() before writing commands.");
        let filename = self
            .module
            .as_ref()
            .expect("Target module not set. Call set_module_name() before writing commands.");

        match command {
            "add" | "sub" | "and" | "or" => {
                let operator = match command {
                    "add" => "+",
                    "sub" => "-",
                    "and" => "&",
                    "or" => "|",
                    _ => unreachable!("Only binary operations should reach here."),
                };
                write!(
                    out_file,
                    "\
                        @SP\n\
                        AM=M-1\n\
                        D=M\n\
                        A=A-1\n\
                        M=M{operator}D\n\
                    "
                )
                .unwrap();
            }
            "neg" | "not" => {
                let operator = match command {
                    "neg" => "-",
                    "not" => "!",
                    _ => unreachable!("Only unary operations should reach here."),
                };
                write!(
                    out_file,
                    "\
                        @SP\n\
                        A=M-1\n\
                        M={operator}M\n\
                    "
                )
                .unwrap();
            }
            "eq" | "gt" | "lt" => {
                let branch = command;
                let id = self.jump_counter;
                self.jump_counter += 1;
                let jump_instruction = match command {
                    "eq" => "JEQ",
                    "gt" => "JGT",
                    "lt" => "JLT",
                    _ => unreachable!("Only eq, gt, lt commands should be reachable."),
                };

                write!(
                    out_file,
                    "\
                        @SP\n\
                        AM=M-1\n\
                        D=M\n\
                        A=A-1\n\
                        D=M-D\n\
                        @__{filename}_{branch}_{id}_true\n\
                        D;{jump_instruction}\n\
                        D=0\n\
                        @__{filename}_{branch}_{id}_end\n\
                        0;JMP\n\
                        (__{filename}_{branch}_{id}_true)\n\
                        D=-1\n\
                        (__{filename}_{branch}_{id}_end)\n\
                        @SP\n\
                        A=M-1\n\
                        M=D\n\
                    "
                )
                .unwrap();
            }
            c => panic!("Non-arithmetic command {c} encountered."),
        }
    }

    pub fn write_pushpop(&mut self, command: CommandType, segment: &str, index: u16) {
        use CommandType::*;

        let out_file = self
            .output_file
            .as_mut()
            .expect("Target file not set. Call set_filename() before writing commands.");
        let filename = self
            .module
            .as_ref()
            .expect("Target module not set. Call set_module_name() before writing commands.");

        /*
         * Predefined Symbols:                     |  Segment Types:
         * SP   0                                  |  argument ARG
         * LCL  1                                  |  local    LCL
         * ARG  2                                  |  static
         * THIS 3                                  |  this     THIS
         * THAT 4                                  |  that     THAT
         * RAM[ 5 - 12]: contains temp segment     |  temp
         * RAM[13 - 15]: general purpose registers |
         *                                         |  pointer
         *                                         |  constant
         */
        match command {
            Push => {
                match segment {
                    "constant" => {
                        let push_code = Self::get_push_code();
                        write!(
                            out_file,
                            "\
                                @{index}\n\
                                D=A\n\
                                {push_code}"
                        )
                        .unwrap();
                    }
                    "argument" | "local" | "this" | "that" => {
                        let seg_symbol = Self::get_segment_symbol(segment);
                        let push_code = Self::get_push_code();

                        write!(
                            out_file,
                            "\
                                @{seg_symbol}\n\
                                D=M\n\
                                @{index}\n\
                                A=A+D\n\
                                D=M\n\
                                {push_code}"
                        )
                        .unwrap();
                    }
                    "pointer" | "temp" => {
                        let seg_symbol = match segment {
                            "pointer" => ["THIS", "THAT"][index as usize],
                            "temp" => ["R5", "R6", "R7", "R8", "R9", "R10", "R11", "R12"][index as usize],
                            _ => unreachable!("Any segment other than \"pointer\" or \"temp\" should't reach here."),
                        };

                        let push_code = Self::get_push_code();

                        write!(
                            out_file,
                            "\
                                @{seg_symbol}\n\
                                D=M\n\
                                {push_code}"
                        )
                        .unwrap();
                    }
                    "static" => {
                        let static_symbol = Self::get_static_symbol(filename, index);
                        let push_code = Self::get_push_code();

                        write!(
                            out_file,
                            "\
                                @{static_symbol}\n\
                                D=M\n\
                                {push_code}"
                        )
                        .unwrap();
                    }
                    s => panic!("Invalid segment '{s}' encountered"),
                }
            }
            Pop => match segment {
                "constant" => {
                    panic!("the \"constant\" segment is virtual. It cannot be written to.")
                }
                "argument" | "local" | "this" | "that" => {
                    let seg_symbol = Self::get_segment_symbol(segment);

                    write!(
                        out_file,
                        "\
                                @{seg_symbol}\n\
                                D=M\n\
                                @{index}\n\
                                D=D+A\n\
                                @R13\n\
                                M=D\n\
                                @SP\n\
                                AM=M-1\n\
                                D=M\n\
                                @R13\n\
                                A=M\n\
                                M=D\n\
                            "
                    )
                    .unwrap();
                }
                "pointer" | "temp" => {
                    let seg_symbol = match segment {
                            "pointer" => ["THIS", "THAT"][index as usize],
                            "temp" => ["R5", "R6", "R7", "R8", "R9", "R10", "R11", "R12"][index as usize],
                            _ => unreachable!("Any segment other than \"pointer\" or \"temp\" should't reach here. Encountered {}", segment),
                        };

                    write!(
                        out_file,
                        "\
                                @SP\n\
                                AM=M-1\n\
                                D=M\n\
                                @{seg_symbol}\n\
                                M=D\n\
                            "
                    )
                    .unwrap();
                }
                "static" => {
                    let static_symbol = Self::get_static_symbol(filename, index);

                    write!(
                        out_file,
                        "\
                                @SP\n\
                                AM=M-1\n\
                                D=M\n\
                                @{static_symbol}\n\
                                M=D\n\
                            "
                    )
                    .unwrap();
                }
                s => panic!("Invalid segment {} encountered.", s),
            },
            _ => panic!("Invalid command {:?} for write_pushpop. This function only concerns push and pop commands.", command),
        }
    }

    pub fn write_label(&mut self, label: &str) {
        let out_file = self
            .output_file
            .as_mut()
            .expect("Target file not set. Call set_filename() before writing commands.");
        let func_name = self
            .module
            .as_ref()
            .expect("Target module not set. Call set_module_name() before writing commands.");

        if !Self::is_valid_label(label) {
            panic!("The label {label} is not valid.");
        }

        // labels are scoped inside a function, therefore we decorate given label with the current function name
        let function_local_label = Self::get_function_label(&func_name, label);
        writeln!(out_file, "({function_local_label})").unwrap();
    }

    pub fn write_goto(&mut self, label: &str) {
        let out_file = self
            .output_file
            .as_mut()
            .expect("Target file not set. Call set_filename() before writing commands.");
        let func_name = self
            .module
            .as_ref()
            .expect("Target module not set. Call set_module_name() before writing commands.");

        if !Self::is_valid_label(label) {
            panic!("The label {label} is not valid.");
        }

        let function_local_label = Self::get_function_label(&func_name, label);
        write!(
            out_file,
            "\
                @{function_local_label}\n\
                0;JMP\n\
            "
        )
        .unwrap();
    }

    pub fn write_if(&mut self, label: &str) {
        let out_file = self
            .output_file
            .as_mut()
            .expect("Target file not set. Call set_filename() before writing commands.");
        let func_name = self
            .module
            .as_ref()
            .expect("Target module not set. Call set_module_name() before writing commands.");

        if !Self::is_valid_label(label) {
            panic!("The label {label} is not valid.");
        }

        let function_local_label = Self::get_function_label(&func_name, label);
        write!(
            out_file,
            "\
                @SP\n\
                AM=M-1\n\
                D=M\n\
                @{function_local_label}\n\
                D;JNE\n\
            "
        )
        .unwrap();
    }

    pub fn write_call(&mut self, function_name: &str, args_count: u16) {
        let out_file = self
            .output_file
            .as_mut()
            .expect("Target file not set. Call set_filename() before writing commands.");

        self.return_counter += 1;
        let return_address_label = Self::get_return_symbol(function_name, self.return_counter);

        write!(
            out_file,
            "\
                @{return_address_label}\n\
                D=A\n\
                {}",
            Self::get_push_code()
        )
        .unwrap();

        Self::push_symbol(out_file, "LCL");
        Self::push_symbol(out_file, "ARG");
        Self::push_symbol(out_file, "THIS");
        Self::push_symbol(out_file, "THAT");

        // ARG = SP-n-5,
        // LCL = SP
        write!(
            out_file,
            "\
                @SP\n\
                D=M\n\
                @LCL\n\
                M=D\n\
                @5\n\
                D=D-A\n\
                @{args_count}\n\
                D=D-A\n\
                @ARG\n\
                M=D\n\
                @{function_name}\n\
                0;JMP\n\
                ({return_address_label})\n\
            "
        )
        .unwrap();
    }

    pub fn write_function(&mut self, function_name: &str, locals_count: u16) {
        let out_file = self
            .output_file
            .as_mut()
            .expect("Target file not set. Call set_module_name() before writing commands.");

        writeln!(out_file, "({function_name})").unwrap();

        // push 0's `locals_count` times
        if locals_count >= 1 {
            write!(
                out_file,
                "\
                    @SP\n\
                    A=M\n\
                "
            )
            .unwrap();

            // iterate (n - 1) times
            // push 0 n times, incrementing the A register
            for _ in 1..locals_count {
                write!(
                    out_file,
                    "\
                        M=0\n\
                        A=A+1\n\
                    "
                )
                .unwrap();
            }

            // push the last 0,
            // increment the stack pointer,
            // then update @SP
            write!(
                out_file,
                "\
                    M=0\n\
                    D=A+1\n\
                    @SP\n\
                    M=D\n\
                "
            )
            .unwrap();
        }
    }

    // FRAME = LCL
    // RET = *(FRAME - 5)
    // *ARG = pop() <- When the function returns, the return value should be at the top of the
    // stack. Since ARG is the top of the current function's call frame, the return value should be
    // put here.
    // SP = ARG + 1 <- Since ARG is the address of the return value of this function,
    // ARG + 1 will be the new SP.
    // THAT = *(FRAME - 1)
    // THIS = *(FRAME - 2)
    // ARG = *(FRAME - 3)
    // LCL = *(FRAME - 4)
    // goto RET
    pub fn write_return(&mut self) {
        let out_file = self
            .output_file
            .as_mut()
            .expect("Target file not set. Call set_filename() before writing commands.");

        write!(
            out_file,
            "\
                // ----------- return -------------\n\
                @LCL\n\
                D=M\n\
                @R13\n\
                M=D\n\
\n\
                @5\n\
                A=D-A\n\
                D=M\n\
                @R14\n\
                M=D\n\
\n\
                @SP\n\
                AM=M-1\n\
                D=M\n\
                @ARG\n\
                A=M\n\
                M=D\n\
\n\
                @ARG\n\
                D=M+1\n\
                @SP\n\
                M=D\n\
\n\
                @R13\n\
                AM=M-1\n\
                D=M\n\
                @THAT\n\
                M=D\n\
\n\
                @R13\n\
                AM=M-1\n\
                D=M\n\
                @THIS\n\
                M=D\n\
\n\
                @R13\n\
                AM=M-1\n\
                D=M\n\
                @ARG\n\
                M=D\n\
\n\
                @R13\n\
                AM=M-1\n\
                D=M\n\
                @LCL\n\
                M=D\n\
\n\
                @R14\n\
                A=M\n\
                0;JMP\n\
                // ----------- return finish -------------\n\
            "
        )
        .unwrap();
    }

    fn push_symbol(out_file: &mut File, symbol: &str) {
        let push_code = Self::get_push_code();

        write!(
            out_file,
            "\
                @{symbol}\n\
                D=M\n\
                {push_code}"
        )
        .unwrap();
    }

    fn get_segment_symbol(segment: &str) -> &'static str {
        match segment {
            "argument" => "ARG",
            "local" => "LCL",
            "this" => "THIS",
            "that" => "THAT",
            _ => panic!("Invalid segment name {} encountered.", segment),
        }
    }

    fn get_static_symbol(filename: &str, index: u16) -> String {
        let basename = filename
            .strip_suffix(".vm")
            .expect("Filenames should end with .vm");
        format!("{basename}.{index}")
    }

    fn get_return_symbol(function_name: &str, counter: u16) -> String {
        format!("{function_name}__return_{}", counter)
    }

    fn get_function_label(function_name: &str, label: &str) -> String {
        format!("{function_name}_local__{label}")
    }

    // Push content in D register on top of stack and increment SP by 1
    fn get_push_code() -> &'static str {
        "\
            @SP\n\
            A=M\n\
            M=D\n\
            @SP\n\
            M=M+1\n\
        "
    }

    fn is_valid_label(label: &str) -> bool {
        if label.len() == 0 {
            return false;
        }

        let mut chars = label.chars();

        if let Some(c) = chars.next() {
            if c.is_ascii_digit() {
                return false;
            }
        }

        for c in chars {
            if !(c.is_ascii_alphanumeric() || c == '.' || c == ':' || c == '_') {
                return false;
            }
        }

        true
    }
}
