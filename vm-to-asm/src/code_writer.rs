use std::{fs::File, io::Write};

use crate::command::CommandType;

pub struct CodeWriter {
    file: Option<(File, String)>,
}

impl CodeWriter {
    pub fn new() -> Self {
        CodeWriter { file: None }
    }

    pub fn set_filename(&mut self, name: &str) {
        let file = File::create(name).expect("set_filename: file not found");
        self.file = Some((file, name.to_string()));
    }

    pub fn close(&mut self) {
        self.file = None;
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
                write!(
                    file,
                    "\
                        @SP\n\
                        AM=M-1\n\
                        D=M\n\
                        A=A-1\n\
                        D=M-D\n\
                        @__{filename}_{branch}_true\n\
                        D;JEQ\n\
                        D=0\n\
                        @__{filename}_{branch}_end\n\
                        0;JMP\n\
                        (__{filename}_{branch}_true)\n\
                        D=-1\n\
                        (__{filename}_{branch}_end)\n\
                        @SP\n\
                        A=M-1\n\
                        M=D\n\
                    "
                )
                .unwrap();
            }
            _ => todo!(),
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
            Arithmetic => panic!("Use the write_arithmetic function for arithmetic commands."),
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
                                A=A+M\n\
                                D=M\n\
                                {push_code}"
                        )
                        .unwrap();
                    }
                    "pointer" | "temp" => {
                        let seg_symbol = match segment {
                        "pointer " => ["THIS", "THAT"][index as usize],
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
                    _ => todo!(),
                }
            }
            Pop => {
                match segment {
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
                                // store the address in a R13\n\
                                @R13\n\
                                M=D\n\
                                // Put the top of the stack in D, and decrement stack\n\
                                @SP\n\
                                AM=M-1\n\
                                D=M\n\
                                // Write D to the address referenced by R13\n\
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
                    _ => todo!(),
                }
            }
            Label => todo!(),
            Goto => todo!(),
            If => todo!(),
            Function => todo!(),
            Return => todo!(),
            Call => todo!(),
        }
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
        format!("__{filename}_static_{index}")
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_write_arithmetic() {
        let mut cw = CodeWriter::new();
        cw.set_filename("output.asm");
        cw.write_pushpop(CommandType::Push, "constant", 3);
        cw.write_pushpop(CommandType::Push, "constant", 3);
        cw.write_arithemtic("lt");
        cw.close();
    }
}
