use std::collections::HashMap;

use crate::{
    instruction::{CommandDest, CommandJump, Instruction, CompValue, Computation},
    token::Token,
};

pub struct Pass;

impl Pass {
    // associate line numbers with each instruction, starting from 0
    pub fn line_resolution(insts: &Vec<Instruction>) -> Vec<(u16, Instruction)> {
        let mut i = 0;
        let mut line = 0;

        let mut line_insts = Vec::new();

        while i < insts.len() {
            if let Instruction::Label(_) = insts[i] {
                let start = i;
                while i < insts.len() {
                    match insts[i] {
                        Instruction::Label(_) => i += 1,
                        _ => break,
                    }
                }

                if i >= insts.len() {
                    panic!("Reached end of file while resolving line for address.");
                }

                i += 1;
                for l in start..i {
                    line_insts.push((line, insts[l].clone()));
                }
                line += 1;
            } else {
                line_insts.push((line, insts[i].clone()));
                line += 1;
                i += 1;
            }
        }

        line_insts
    }

    pub fn symbol_resolution(insts: &Vec<Instruction>) -> HashMap<String, u16> {
        use Token::*;

        let mut variable_address = 0x10;
        let line_insts = Self::line_resolution(insts);
        let mut table = HashMap::from([
            ("SP".into(), 0x0),
            ("LCL".into(), 0x1),
            ("ARG".into(), 0x2),
            ("THIS".into(), 0x3),
            ("THAT".into(), 0x4),
            ("R0".into(), 0x0),
            ("R1".into(), 0x1),
            ("R2".into(), 0x2),
            ("R3".into(), 0x3),
            ("R4".into(), 0x4),
            ("R5".into(), 0x5),
            ("R6".into(), 0x6),
            ("R7".into(), 0x7),
            ("R8".into(), 0x8),
            ("R9".into(), 0x9),
            ("R10".into(), 0xa),
            ("R11".into(), 0xb),
            ("R12".into(), 0xc),
            ("R13".into(), 0xd),
            ("R14".into(), 0xe),
            ("R15".into(), 0xf),
            ("SCREEN".into(), 0x4000),
            ("KBD".into(), 0x6000),
        ]);

        // list of label names and their line numbers.
        // we want to populate the table before running the actual pass
        // because we don't want to confuse labels with new variables.
        // note htat labels can appear after a new variable, meaning that
        // resolution would be ambiuous without this pre-population
        let labels = line_insts.iter().filter_map(|(l, inst)| {
            if let Instruction::Label(ref name) = inst {
                Some((name.clone(), *l))
            } else {
                None
            }
        });
        table.extend(labels);

        for inst in &line_insts {
            match inst {
                (_, Instruction::Address(name)) => match name {
                    Number(_) => (),
                    Symbol(name) => {
                        variable_address = Self::handle_symbol(&mut table, name, variable_address);
                    }
                    // NOTE: registers like A, D, M are NOT treated specially here, so we must
                    // manually put them in the table
                    RegA | RegM | RegD | RegAM | RegAD | RegMD | RegAMD | JGT | JEQ | JGE | JLT
                    | JNE | JLE | JMP => {
                        let name = match name {
                            RegA => "RegA",
                            RegM => "RegM",
                            RegD => "RegD",
                            RegAM => "RegAM",
                            RegAD => "RegAD",
                            RegMD => "RegMD",
                            RegAMD => "RegAMD",
                            JGT => "JGT",
                            JEQ => "JEQ",
                            JGE => "JGE",
                            JLT => "JLT",
                            JNE => "JNE",
                            JLE => "JLE",
                            JMP => "JMP",
                            _ => unreachable!(),
                        };
                        variable_address = Self::handle_symbol(&mut table, name, variable_address);
                    }
                    _ => (),
                },
                _ => (),
            }
        }

        table
    }

    pub fn translate(insts: &Vec<Instruction>) -> Vec<String> {
        use Token::*;

        let table = Self::symbol_resolution(insts);
        let mut machine_insts = Vec::new();

        for inst in insts {
            match inst {
                Instruction::Address(name) => match name {
                    Token::Number(num) => {
                        let minst = format!("0{:015b}", num);
                        machine_insts.push(minst);
                    }
                    Symbol(name) => {
                        let address = table.get(name).expect("Symbol not found from table!");
                        let minst = format!("0{:015b}", address);
                        machine_insts.push(minst);
                    }
                    RegA | RegM | RegD | RegAM | RegAD | RegMD | RegAMD | JGT | JEQ | JGE | JLT
                    | JNE | JLE | JMP => {
                        let name = match name {
                            RegA => "RegA",
                            RegM => "RegM",
                            RegD => "RegD",
                            RegAM => "RegAM",
                            RegAD => "RegAD",
                            RegMD => "RegMD",
                            RegAMD => "RegAMD",
                            JGT => "JGT",
                            JEQ => "JEQ",
                            JGE => "JGE",
                            JLT => "JLT",
                            JNE => "JNE",
                            JLE => "JLE",
                            JMP => "JMP",
                            _ => unreachable!(),
                        };
                        let address = table.get(name).expect("Symbol not found from table!");
                        let minst = format!("0{:015b}", address);
                        machine_insts.push(minst);
                    }
                    _ => panic!("Only symbols and numbers are allowed in A-instructions."),
                },
                Instruction::Command(dest, comp, jump) => {
                    let dest = Self::encode_dest(dest);
                    let comp = Self::encode_comp(comp);
                    let jump = Self::encode_jump(jump);

                    let minst = format!("111{}{:03b}{:03b}", comp, dest, jump);
                    machine_insts.push(minst);
                }
                // skip label
                Instruction::Label(_) => (),
            }
        }

        machine_insts
    }

    // return new address if inserted
    fn handle_symbol(table: &mut HashMap<String, u16>, name: &str, addr: u16) -> u16 {
        if !table.contains_key(name) {
            table.insert(name.into(), addr);
            addr + 1
        } else {
            addr
        }
    }

    fn encode_dest(dest: &CommandDest) -> u8 {
        match dest {
            CommandDest::NULL => 0,
            CommandDest::M => 1,
            CommandDest::D => 2,
            CommandDest::MD => 3,
            CommandDest::A => 4,
            CommandDest::AM => 5,
            CommandDest::AD => 6,
            CommandDest::AMD => 7,
        }
    }

    fn encode_comp(comp: &Computation) -> &'static str {
        use Computation::*;
        use CompValue::*;

        match comp {
            Literal(Zero) => "0101010",
            Literal(One) =>  "0111111",
            Negative(One) => "0111010",
            Literal(RegD) => "0001100",
            Literal(RegA) => "0110000",
            Literal(RegM) => "1110000",
            Not(RegD) => "0001101",
            Not(RegA) => "0110001",
            Not(RegM) => "1110001",
            Negative(RegD) => "0001111",
            Negative(RegA) => "0110011",
            Negative(RegM) => "1110011",
            Add {lhs: RegD, rhs: One} => "0011111",
            Add {lhs: RegA, rhs: One} => "0110111",
            Add {lhs: RegM, rhs: One} => "1110111",
            Sub {lhs: RegD, rhs: One} => "0001110",
            Sub {lhs: RegA, rhs: One} => "0110010",
            Sub {lhs: RegM, rhs: One} => "1110010",
            Add {lhs: RegD, rhs: RegA} => "0000010",
            Add {lhs: RegD, rhs: RegM} => "1000010",
            Sub {lhs: RegD, rhs: RegA} => "0010011",
            Sub {lhs: RegD, rhs: RegM} => "1010011",
            Sub {lhs: RegA, rhs: RegD} => "0000111",
            Sub {lhs: RegM, rhs: RegD} => "1000111",
            And {lhs: RegD, rhs: RegA} => "0000000",
            And {lhs: RegD, rhs: RegM} => "1000000",
            Or {lhs: RegA, rhs: RegD} => "0010101",
            Or {lhs: RegD, rhs: RegM} => "1010101",
            _ => panic!("Invalid operation encountered: {:?}", comp),
        }
    }

    fn encode_jump(jump: &CommandJump) -> u8 {
        match jump {
            CommandJump::NULL => 0,
            CommandJump::JGT => 1,
            CommandJump::JEQ => 2,
            CommandJump::JGE => 3,
            CommandJump::JLT => 4,
            CommandJump::JNE => 5,
            CommandJump::JLE => 6,
            CommandJump::JMP => 7,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Pass;
    use crate::instruction::*;
    use crate::parser::Parser;
    use crate::token::Token;

    #[test]
    fn test_line_resolution1() {
        use Token::*;

        let input = "
            @i
            (LOOP)
            @j
            D=M
            (END)
            A=D
        ";
        let mut parser = Parser::new(input);
        let insts = parser.parse();
        let line_insts = Pass::line_resolution(&insts);
        let expected = vec![
            (0, Instruction::Address(Symbol("i".into()))),
            (1, Instruction::Label("LOOP".into())),
            (1, Instruction::Address(Symbol("j".into()))),
            (
                2,
                Instruction::Command(
                    CommandDest::D,
                    Computation::Literal(CompValue::RegM),
                    CommandJump::NULL,
                ),
            ),
            (3, Instruction::Label("END".into())),
            (
                3,
                Instruction::Command(
                    CommandDest::A,
                    Computation::Literal(CompValue::RegD),
                    CommandJump::NULL,
                ),
            ),
        ];
        assert_eq!(line_insts, expected);
    }

    #[test]
    fn test_line_resolution2() {
        use Token::*;

        let input = "
            (LOOP)
            @j
            D=M
            (END)
            A=D
        ";
        let mut parser = Parser::new(input);
        let insts = parser.parse();
        let line_insts = Pass::line_resolution(&insts);
        let expected = vec![
            (0, Instruction::Label("LOOP".into())),
            (0, Instruction::Address(Symbol("j".into()))),
            (
                1,
                Instruction::Command(
                    CommandDest::D,
                    Computation::Literal(CompValue::RegM),
                    CommandJump::NULL,
                ),
            ),
            (2, Instruction::Label("END".into())),
            (
                2,
                Instruction::Command(
                    CommandDest::A,
                    Computation::Literal(CompValue::RegD),
                    CommandJump::NULL,
                ),
            ),
        ];
        assert_eq!(line_insts, expected);
    }

    #[test]
    fn test_line_resolution3() {
        use Token::*;

        let input = "
            (LOOP)
            (LOOP2)
            @j
            D=M
            (END)
            A=D
        ";
        let mut parser = Parser::new(input);
        let insts = parser.parse();
        let line_insts = Pass::line_resolution(&insts);
        let expected = vec![
            (0, Instruction::Label("LOOP".into())),
            (0, Instruction::Label("LOOP2".into())),
            (0, Instruction::Address(Symbol("j".into()))),
            (
                1,
                Instruction::Command(
                    CommandDest::D,
                    Computation::Literal(CompValue::RegM),
                    CommandJump::NULL,
                ),
            ),
            (2, Instruction::Label("END".into())),
            (
                2,
                Instruction::Command(
                    CommandDest::A,
                    Computation::Literal(CompValue::RegD),
                    CommandJump::NULL,
                ),
            ),
        ];
        assert_eq!(line_insts, expected);
    }

    #[test]
    fn test_foobar() {
        let input = "
            @i
            M=1
            @sum
            M=0
            (LOOP)
            @i
            D=M
            @100
            D=D-A
            @END
            D;JGT
            @i
            D=M
            @sum
            M=D+M
            @i
            M=M+1
            @LOOP
            0;JMP
            (END)
            @END
            0;JMP
        ";
        let mut parser = Parser::new(input);
        let insts = parser.parse();
        let minsts = Pass::translate(&insts);
        println!("{}", minsts.join("\n"));
    }
}
