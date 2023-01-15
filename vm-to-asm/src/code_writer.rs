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
            "add" => {
                let asm = "\
                    @SP\n\
                    AM=M-1\n\
                    D=M\n\
                    A=A-1\n\
                    M=D+M\n\
                ";
                write!(file, "{}", asm).unwrap();
            }
            "sub" => {
                let asm = "\
                    @SP\n\
                    AM=M-1\n\
                    D=M\n\
                    A=A-1\n\
                    M=M-D\n\
                ";
                write!(file, "{}", asm).unwrap();
            }
            "neg" => {
                let asm = "\
                    @SP\n\
                    A=M-1\n\
                    M=-M\n\
                ";
                write!(file, "{}", asm).unwrap();
            }
            "eq" => {
                write!(
                    file,
                    "\
                    @SP\n\
                    AM=M-1\n\
                    D=M\n\
                    A=A-1\n\
                    D=M-D\n\
                    @__{filename}_eq_true\n\
                    D;JEQ\n\
                    D=0\n\
                    @__{filename}_eq_end\n\
                    0;JMP\n\
                    (__{filename}_eq_true)\n\
                    D=-1\n\
                    (__{filename}_eq_end)\n\
                    @SP\n\
                    A=M-1\n\
                    M=D\n\
                "
                )
                .unwrap();
            }
            "gt" => {
                write!(
                    file,
                    "\
                    @SP\n\
                    AM=M-1\n\
                    D=M\n\
                    A=A-1\n\
                    D=M-D\n\
                    @__{filename}_gt_true\n\
                    D;JGT\n\
                    D=0\n\
                    @__{filename}_gt_end\n\
                    0;JMP\n\
                    (__{filename}_gt_true)\n\
                    D=-1\n\
                    (__{filename}_gt_end)\n\
                    @SP\n\
                    A=M-1\n\
                    M=D\n\
                "
                )
                .unwrap();
            }
            "lt" => {
                write!(
                    file,
                    "\
                    @SP\n\
                    AM=M-1\n\
                    D=M\n\
                    A=A-1\n\
                    D=M-D\n\
                    @__{filename}_lt_true\n\
                    D;JLT\n\
                    D=0\n\
                    @__{filename}_lt_end\n\
                    0;JMP\n\
                    (__{filename}_lt_true)\n\
                    D=-1\n\
                    (__{filename}_lt_end)\n\
                    @SP\n\
                    A=M-1\n\
                    M=D\n\
                "
                )
                .unwrap();
            }
            "and" => {
                let asm = "\
                    @SP\n\
                    AM=M-1\n\
                    D=M\n\
                    A=A-1\n\
                    M=M&D\n\
                ";
                write!(file, "{}", asm).unwrap();
            }
            "or" => {
                let asm = "\
                    @SP\n\
                    AM=M-1\n\
                    D=M\n\
                    A=A-1\n\
                    M=M|D\n\
                ";
                write!(file, "{}", asm).unwrap();
            }
            "not" => {
                let asm = "\
                    @SP\n\
                    A=M-1\n\
                    M=!M\n\
                ";
                write!(file, "{}", asm).unwrap();
            }
            _ => todo!(),
        }
    }

    pub fn write_pushpop(&mut self, command: CommandType, segment: &str, index: u16) {
        use CommandType::*;

        let (file, _filename) = match self.file {
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
            Push => match segment {
                "constant" => {
                    write!(
                        file,
                        "\
                            @{index}\n\
                            D=A\n\
                            @SP\n\
                            A=M\n\
                            M=D\n\
                            @SP\n\
                            M=M+1\n\
                        "
                    )
                    .unwrap();
                }
                "argument" | "local" | "this" | "that" | "temp" => {
                    let seg_symbol = Self::get_segment_symbol(segment, index);

                    write!(
                        file,
                        "\
                            @{seg_symbol}\n\
                            D=M\n\
                            @{index}\n\
                            A=A+M\n\
                            D=M\n\
                            @SP\n\
                            A=M\n\
                            M=D\n\
                            D=A+1\n\
                            @SP\n\
                            M=D\n\
                        "
                    )
                    .unwrap();
                }
                "pointer" => {
                    let seg_symbol = match index {
                        0 => "THIS",
                        1 => "THAT",
                        _ => panic!(
                            "Index to pointer segment must be 0 or 1. Received: {}",
                            index
                        ),
                    };

                    write!(
                        file,
                        "\
                            @{seg_symbol}\n\
                            D=M\n\
                            @SP\n\
                            A=M\n\
                            M=D\n\
                            D=A+1\n\
                            @SP\n\
                            M=D\n\
                        "
                    )
                    .unwrap();
                },
                "static" => {

                },
                _ => todo!(),
            },
            Pop => todo!(),
            Label => todo!(),
            Goto => todo!(),
            If => todo!(),
            Function => todo!(),
            Return => todo!(),
            Call => todo!(),
        }
    }

    fn get_segment_symbol(segment: &str, index: u16) -> &'static str {
        match segment {
            "argument" => "ARG",
            "local" => "LCL",
            "this" => "THIS",
            "that" => "THAT",
            "temp" => {
                // temp starts at R5 ~ R12
                // if index == 0, then R5, == 1, then R6... and so on
                ["R5", "R6", "R7", "R8", "R9", "R10", "R11", "R12"][index as usize]
            }
            _ => todo!(),
        }
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
