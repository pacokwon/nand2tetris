use crate::instruction::{CommandDest, CommandJump, CompValue, Computation, Instruction};
use crate::lexer::Lexer;
use crate::token::Token;

pub struct Parser {
    lines: Vec<Vec<Token>>,
}

impl Parser {
    pub fn new(input: &str) -> Self {
        let mut lexer = Lexer::new(input);
        let lines = Self::split_at_newline(lexer.all_tokens());
        Parser { lines }
    }

    fn split_at_newline(tokens: Vec<Token>) -> Vec<Vec<Token>> {
        let mut groups = Vec::new();

        let mut i = 0;
        while i < tokens.len() {
            let mut group = Vec::new();

            // skip newlines
            while i < tokens.len() {
                if let Token::Newline = &tokens[i] {
                    i += 1;
                } else {
                    break;
                }
            }

            while i < tokens.len() {
                match &tokens[i] {
                    Token::Newline => {
                        break;
                    }
                    Token::Eof => {
                        i += 1;
                        break;
                    }
                    _ => {
                        group.push(tokens[i].clone());
                        i += 1;
                    }
                }
            }

            if !group.is_empty() {
                groups.push(group);
            }
        }

        groups
    }

    pub fn parse(&mut self) -> Vec<Instruction> {
        let mut parsed = Vec::new();

        for line in &self.lines {
            let inst = Self::parse_line(line);
            parsed.push(inst);
        }

        parsed
    }

    pub fn parse_line(line: &Vec<Token>) -> Instruction {
        use Token::*;

        match line[0] {
            // a-isntruction
            At => match line[1] {
                RegA | RegM | RegD | RegAM | RegAD | RegMD | RegAMD | Symbol(_) | Number(_) => Instruction::Address(line[1].clone()),
                _ => panic!("Unexpected token {:?} after '@'; only symbols and numbers are allowed for A-instructions.", line[1]),
            },
            // address (pseudo) instruction
            LParen => match line[1] {
                Symbol(ref name) => Instruction::Label(name.clone()),
                _ => panic!("Unexpected token {:?} after '('; only symbols are allowed for label instructions.", line[1]),
            }
            // c-instruction
            _ => {
                let (dest, position) = match line[1] {
                    Equal => {
                        let dest = match line[0] {
                            RegM => CommandDest::M,
                            RegD => CommandDest::D,
                            RegA => CommandDest::A,
                            RegMD => CommandDest::MD,
                            RegAM => CommandDest::AM,
                            RegAD => CommandDest::AD,
                            RegAMD => CommandDest::AMD,
                            Symbol(ref name) => panic!("The symbol '{}' cannot be used as a destination.", name),
                            ref t => panic!("Unexpected token {:?} for C-instruction destination; only symbols are allowed for destinations.", t),
                        };
                        (dest, 2)
                    },
                    _ => (CommandDest::NULL, 0),
                };

                // We assume that the computation part always exists
                let (comp, position) = match line[position] {
                    // check if unary
                    Minus => {
                        let comp = Self::parse_comp_value(&line[position + 1]);
                        (Computation::Negative(comp), position + 2)
                    },
                    Bang => {
                        let comp = Self::parse_comp_value(&line[position + 1]);
                        (Computation::Not(comp), position + 2)
                    },
                    // binary operation
                    RegA | RegD | RegM | Number(_) => {
                        if position == line.len() - 1 {
                            let comp = Self::parse_comp_value(&line[position]);
                            (Computation::Literal(comp), position + 1)
                        } else if let Semicolon = line[position + 1] {
                            let comp = Self::parse_comp_value(&line[position]);
                            (Computation::Literal(comp), position + 1)
                        } else {
                            let lhs = Self::parse_comp_value(&line[position]);
                            let rhs = Self::parse_comp_value(&line[position + 2]);

                            let comp = match line[position + 1] {
                                Plus => Computation::Add{lhs, rhs},
                                Minus => Computation::Sub{lhs, rhs},
                                Ampersand => Computation::And{lhs, rhs},
                                Pipe => Computation::Or{lhs, rhs},
                                ref t => panic!("Invalid operator {:?}. An operator must be one of '+', '-', '&', '|'", t),
                            };

                            (comp, position + 3)
                        }
                    },
                    ref t => panic!("A computation must be a unary or binary expression. Encountered: {:?}", t),
                };

                if position >= line.len() {
                    return Instruction::Command(dest, comp, CommandJump::NULL);
                }

                assert_eq!(line[position], Semicolon);
                let position = position + 1;
                let jump = Self::parse_jump(&line[position]);
                Instruction::Command(dest, comp, jump)
            }
        }
    }

    fn parse_comp_value(token: &Token) -> CompValue {
        use Token::*;

        match token {
            RegA => CompValue::RegA,
            RegD => CompValue::RegD,
            RegM => CompValue::RegM,
            Number(0) => CompValue::Zero,
            Number(1) => CompValue::One,
            t => panic!("Unexpected token {:?} in comp value.", t),
        }
    }

    fn parse_jump(token: &Token) -> CommandJump {
        use Token::*;

        match token {
            JGT => CommandJump::JGT,
            JEQ => CommandJump::JEQ,
            JGE => CommandJump::JGE,
            JLT => CommandJump::JLT,
            JNE => CommandJump::JNE,
            JLE => CommandJump::JLE,
            JMP => CommandJump::JMP,
            t => panic!("Unexpected token {:?} while parsing jump token.", t),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Lexer;
    use super::Parser;
    use super::Token;
    use crate::instruction::*;

    #[test]
    fn test_split_at_newline() {
        use Token::*;

        let input = "foobar 30 &|\nfoo\n\nbar\n";
        let groups = Parser::split_at_newline(Lexer::new(input).all_tokens());
        let expected = vec![
            vec![Symbol(String::from("foobar")), Number(30), Ampersand, Pipe],
            vec![Symbol(String::from("foo"))],
            vec![Symbol(String::from("bar"))],
        ];
        assert_eq!(groups, expected);
    }

    #[test]
    fn test_address() {
        let input = "
            @10
            @A
            @R0
        ";
        let mut parser = Parser::new(input);
        let insts = parser.parse();
        let expected = vec![
            Instruction::Address(Token::Number(10)),
            Instruction::Address(Token::RegA),
            Instruction::Address(Token::Symbol(String::from("R0"))),
        ];
        assert_eq!(insts, expected);
    }

    #[test]
    fn test_comp_literal() {
        let input = "
            D=A
            AMD=D
            AMD=D;JMP
        ";
        let mut parser = Parser::new(input);
        let insts = parser.parse();
        let expected = vec![
            Instruction::Command(
                CommandDest::D,
                Computation::Literal(CompValue::RegA),
                CommandJump::NULL,
            ),
            Instruction::Command(
                CommandDest::AMD,
                Computation::Literal(CompValue::RegD),
                CommandJump::NULL,
            ),
            Instruction::Command(
                CommandDest::AMD,
                Computation::Literal(CompValue::RegD),
                CommandJump::JMP,
            ),
        ];
        assert_eq!(insts, expected);
    }

    #[test]
    fn test_comp_unary() {
        let input = "D=-A\nAMD=!D";
        let mut parser = Parser::new(input);
        let insts = parser.parse();
        let expected = vec![
            Instruction::Command(
                CommandDest::D,
                Computation::Negative(CompValue::RegA),
                CommandJump::NULL,
            ),
            Instruction::Command(
                CommandDest::AMD,
                Computation::Not(CompValue::RegD),
                CommandJump::NULL,
            ),
        ];
        assert_eq!(insts, expected);
    }

    #[test]
    fn test_comp_binary() {
        use CompValue::*;

        let input = "D=A+M\nAMD=A&1\nM=M-1\nD=A|M";
        let mut parser = Parser::new(input);
        let insts = parser.parse();
        let expected = vec![
            Instruction::Command(
                CommandDest::D,
                Computation::Add {
                    lhs: RegA,
                    rhs: RegM,
                },
                CommandJump::NULL,
            ),
            Instruction::Command(
                CommandDest::AMD,
                Computation::And {
                    lhs: RegA,
                    rhs: One,
                },
                CommandJump::NULL,
            ),
            Instruction::Command(
                CommandDest::M,
                Computation::Sub {
                    lhs: RegM,
                    rhs: One,
                },
                CommandJump::NULL,
            ),
            Instruction::Command(
                CommandDest::D,
                Computation::Or {
                    lhs: RegA,
                    rhs: RegM,
                },
                CommandJump::NULL,
            ),
        ];
        assert_eq!(insts, expected);
    }

    #[test]
    fn test_comp_jump() {
        use CompValue::*;

        let input = "
            0;JGT
            D-1;JEQ
            A&1;JLT
            A&M;JNE
            D|0;JLE
            M+1;JMP
        ";
        let mut parser = Parser::new(input);
        let insts = parser.parse();
        let expected = vec![
            Instruction::Command(
                CommandDest::NULL,
                Computation::Literal(Zero),
                CommandJump::JGT,
            ),
            Instruction::Command(
                CommandDest::NULL,
                Computation::Sub {
                    lhs: RegD,
                    rhs: One,
                },
                CommandJump::JEQ,
            ),
            Instruction::Command(
                CommandDest::NULL,
                Computation::And {
                    lhs: RegA,
                    rhs: One,
                },
                CommandJump::JLT,
            ),
            Instruction::Command(
                CommandDest::NULL,
                Computation::And {
                    lhs: RegA,
                    rhs: RegM,
                },
                CommandJump::JNE,
            ),
            Instruction::Command(
                CommandDest::NULL,
                Computation::Or {
                    lhs: RegD,
                    rhs: Zero,
                },
                CommandJump::JLE,
            ),
            Instruction::Command(
                CommandDest::NULL,
                Computation::Add {
                    lhs: RegM,
                    rhs: One,
                },
                CommandJump::JMP,
            ),
        ];
        assert_eq!(insts, expected);
    }

    #[test]
    fn test_comp_mixed() {
        use CompValue::*;

        let input = "
            A=0;JGT
            AMD=D-1;JEQ
            AD=A&1;JLT
            AM=A&M;JNE
            D=D|0;JLE
            M=M+1;JMP
        ";
        let mut parser = Parser::new(input);
        let insts = parser.parse();
        let expected = vec![
            Instruction::Command(CommandDest::A, Computation::Literal(Zero), CommandJump::JGT),
            Instruction::Command(
                CommandDest::AMD,
                Computation::Sub {
                    lhs: RegD,
                    rhs: One,
                },
                CommandJump::JEQ,
            ),
            Instruction::Command(
                CommandDest::AD,
                Computation::And {
                    lhs: RegA,
                    rhs: One,
                },
                CommandJump::JLT,
            ),
            Instruction::Command(
                CommandDest::AM,
                Computation::And {
                    lhs: RegA,
                    rhs: RegM,
                },
                CommandJump::JNE,
            ),
            Instruction::Command(
                CommandDest::D,
                Computation::Or {
                    lhs: RegD,
                    rhs: Zero,
                },
                CommandJump::JLE,
            ),
            Instruction::Command(
                CommandDest::M,
                Computation::Add {
                    lhs: RegM,
                    rhs: One,
                },
                CommandJump::JMP,
            ),
        ];
        assert_eq!(insts, expected);
    }

    #[test]
    fn test_mixed() {
        use CompValue::*;

        let input = "
            @30
            A=0;JGT
            AMD=D-1;JEQ
            (LOOP)
            AD=A&1;JLT
            AM=A&M;JNE
            D=D|0;JLE
            M=M+1;JMP
        ";
        let mut parser = Parser::new(input);
        let insts = parser.parse();
        let expected = vec![
            Instruction::Address(Token::Number(30)),
            Instruction::Command(CommandDest::A, Computation::Literal(Zero), CommandJump::JGT),
            Instruction::Command(CommandDest::AMD, Computation::Sub{lhs: RegD, rhs: One}, CommandJump::JEQ),
            Instruction::Label(String::from("LOOP")),
            Instruction::Command(CommandDest::AD, Computation::And{lhs: RegA, rhs: One}, CommandJump::JLT),
            Instruction::Command(CommandDest::AM, Computation::And{lhs: RegA, rhs: RegM}, CommandJump::JNE),
            Instruction::Command(CommandDest::D, Computation::Or{lhs: RegD, rhs: Zero}, CommandJump::JLE),
            Instruction::Command(CommandDest::M, Computation::Add{lhs: RegM, rhs: One}, CommandJump::JMP),
        ];
        assert_eq!(insts, expected);
    }
}
