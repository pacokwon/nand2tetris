use crate::{token::Token, register::Register};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Instruction {
    Address(Token),
    Command(CommandDest, Computation, CommandJump),
    Label(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CommandDest {
    NULL,
    M,
    D,
    MD,
    A,
    AM,
    AD,
    AMD,
}

// computation values are A, D, M, 0, 1, -1
/// they can be used inside computations
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CompValue {
    RegA,
    RegD,
    RegM,
    Zero,
    One,
}

impl From<Register> for CompValue {
    fn from(reg: Register) -> Self {
        match reg {
            Register::A => Self::RegA,
            Register::D => Self::RegD,
            Register::M => Self::RegM,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Computation {
    Literal(CompValue),
    Not(CompValue),
    Negative(CompValue),
    Add{lhs: CompValue, rhs: CompValue},
    Sub{lhs: CompValue, rhs: CompValue},
    And{lhs: CompValue, rhs: CompValue},
    Or{lhs: CompValue, rhs: CompValue},
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CommandJump {
    NULL,
    JGT,
    JEQ,
    JGE,
    JLT,
    JNE,
    JLE,
    JMP,
}
