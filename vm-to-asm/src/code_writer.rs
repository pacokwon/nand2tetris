use std::{fs::File, io::Write};

use crate::command::CommandType;

pub struct CodeWriter {
    file: Option<(File, String)>,
    current_function_name: Option<String>,
    eq_counter: u16,
    gt_counter: u16,
    lt_counter: u16,
}

impl CodeWriter {
    pub fn new() -> Self {
        CodeWriter {
            file: None,
            current_function_name: None,
            eq_counter: 0,
            gt_counter: 0,
            lt_counter: 0,
        }
    }

    pub fn set_filename(&mut self, name: &str) {
        let file = File::create(name).expect("set_filename: file not found");
        self.file = Some((file, name.to_string()));
        self.eq_counter = 0;
        self.gt_counter = 0;
        self.lt_counter = 0;
    }

    pub fn close(&mut self) {
        self.file = None;
    }

    pub fn write_init(&mut self) {
        let (file, _) = match self.file {
            Some(ref mut f) => f,
            None => panic!("Target file not set. Call set_filename() before writing commands."),
        };

        write!(
            file,
            "\
                @256\n\
                D=A\n\
                @SP\n\
                M=D\n\
                @SimpleFunction.test\n\
                0;JMP\n\
            "
        )
        .unwrap();
    }

    pub fn write_arithemtic(&mut self, command: &str) {
        let (file, filename) = match self.file {
            Some(ref mut f) => f,
            None => panic!("Target file not set. Call set_filename() before writing commands."),
        };

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
                    file,
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
                    file,
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
                let (id, jump_instruction) = match command {
                    "eq" => {
                        self.eq_counter += 1;
                        (self.eq_counter, "JEQ")
                    }
                    "gt" => {
                        self.gt_counter += 1;
                        (self.gt_counter, "JGT")
                    }
                    "lt" => {
                        self.lt_counter += 1;
                        (self.lt_counter, "JLT")
                    }
                    _ => unreachable!("Only eq, gt, lt commands should be reachable."),
                };

                write!(
                    file,
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

        let (file, filename) = match self.file {
            Some(ref mut f) => f,
            None => panic!("Target file not set. Call set_filename() before writing commands."),
        };

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
                            file,
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
                            file,
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
                            file,
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
                            file,
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
                        file,
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
                        file,
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
                        file,
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
        let (file, _) = match self.file {
            Some(ref mut f) => f,
            None => panic!("Target file not set. Call set_filename() before writing commands."),
        };

        let func_name = self.current_function_name
            .as_ref()
            .expect("Error! VM is currently not inside a function. The label command requires that it is in a function context.");

        if !Self::is_valid_label(label) {
            panic!("The label {label} is not valid.");
        }

        // labels are scoped inside a function, therefore we decorate given label with the current function name
        let function_local_label = Self::get_function_label(&func_name, label);
        writeln!(file, "({function_local_label})").unwrap();
    }

    pub fn write_goto(&mut self, label: &str) {
        let (file, _) = match self.file {
            Some(ref mut f) => f,
            None => panic!("Target file not set. Call set_filename() before writing commands."),
        };

        let func_name = self.current_function_name
            .as_ref()
            .expect("Error! VM is currently not inside a function. The label command requires that it is in a function context.");

        if !Self::is_valid_label(label) {
            panic!("The label {label} is not valid.");
        }

        let function_local_label = Self::get_function_label(&func_name, label);
        write!(
            file,
            "\
                @{function_local_label}\n\
                0;JMP\n\
            "
        )
        .unwrap();
    }

    pub fn write_if(&mut self, label: &str) {
        let (file, _) = match self.file {
            Some(ref mut f) => f,
            None => panic!("Target file not set. Call set_filename() before writing commands."),
        };

        let func_name = self.current_function_name
            .as_ref()
            .expect("Error! VM is currently not inside a function. The label command requires that it is in a function context.");

        if !Self::is_valid_label(label) {
            panic!("The label {label} is not valid.");
        }

        let function_local_label = Self::get_function_label(&func_name, label);
        write!(
            file,
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
        let (file, _) = match self.file {
            Some(ref mut f) => f,
            None => panic!("Target file not set. Call set_filename() before writing commands."),
        };

        let return_address_label = Self::get_return_symbol(function_name);

        Self::push_symbol(file, &return_address_label);
        Self::push_symbol(file, "LCL");
        Self::push_symbol(file, "ARG");
        Self::push_symbol(file, "THIS");
        Self::push_symbol(file, "THAT");

        // ARG = SP-n-5,
        // LCL = SP
        write!(
            file,
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
            "
        )
        .unwrap();
    }

    pub fn write_function(&mut self, function_name: &str, locals_count: u16) {
        let (file, _) = match self.file {
            Some(ref mut f) => f,
            None => panic!("Target file not set. Call set_filename() before writing commands."),
        };

        writeln!(file, "({function_name})").unwrap();

        // push 0's `locals_count` times
        if locals_count >= 1 {
            write!(
                file,
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
                    file,
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
                file,
                "\
                    M=0\n\
                    D=A+1\n\
                    @SP\n\
                    M=D\n\
                "
            )
            .unwrap();
        }

        // We now enter the function body.
        // set the function name, so that other places where the current function name is needed
        // can utilize it.
        self.current_function_name = Some(function_name.to_string());
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
        let (file, _) = match self.file {
            Some(ref mut f) => f,
            None => panic!("Target file not set. Call set_filename() before writing commands."),
        };

        write!(
            file,
            "\
                @LCL\n\
                D=M\n\
                @R13\n\
                MD=D\n\
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
                0;JMP\n\
            "
        )
        .unwrap();

        // we now exit the function body.
        // unset the function name.
        self.current_function_name = None;
    }

    fn push_symbol(file: &mut File, symbol: &str) {
        let push_code = Self::get_push_code();

        write!(
            file,
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
            .strip_suffix(".asm")
            .expect("Filenames should end with .vm");
        format!("{basename}.{index}")
    }

    fn get_return_symbol(function_name: &str) -> String {
        format!("{function_name}__return")
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
