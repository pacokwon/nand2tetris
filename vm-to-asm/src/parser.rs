use crate::command::CommandType;

pub struct Parser {
    source: Vec<Vec<String>>,
    line: usize,
}

impl Parser {
    pub fn new(input: &str) -> Self {
        let source: Vec<Vec<String>> = input
            .lines()
            .map(|s| {
                s.split_ascii_whitespace()
                    .map(str::to_owned)
                    .collect::<Vec<String>>()
            })
            .filter(|v| !v.is_empty())
            .collect();
        let line = 0;
        Parser { source, line }
    }

    pub fn advance(&mut self) {
        self.line += 1;
    }

    pub fn has_more_commands(&self) -> bool {
        self.line < self.source.len()
    }

    pub fn command_type(&self) -> CommandType {
        let line = &self.source[self.line];

        match line[0].as_str() {
            "add" | "sub" | "neg" | "eq" | "gt" | "lt" | "and" | "or" | "not" => {
                CommandType::Arithmetic
            }
            "push" => CommandType::Push,
            "pop" => CommandType::Pop,
            c => panic!("Invalid command {} at line {}", c, self.line + 1),
        }
    }

    pub fn arg1(&self) -> &str {
        let line = &self.source[self.line];
        match line[0].as_str() {
            "add" | "sub" | "neg" | "eq" | "gt" | "lt" | "and" | "or" | "not" => {
                line[0].as_str()
            }
            "pop" | "push" => {
                line[1].as_str()
            },
            _ => todo!(),
        }
    }

    pub fn arg2(&self) -> u16 {
        let line = &self.source[self.line];
        match line[0].as_str() {
            "add" | "sub" | "neg" | "eq" | "gt" | "lt" | "and" | "or" | "not" => {
                panic!("Invalid call to arg2 for arithmetic operation {}", line[0])
            }
            "pop" | "push" => {
                line[2].parse().expect("Expected integer for second argument.")
            },
            _ => todo!(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn test_parser() {
        let input = "
                add
                sub

                add

                @2
                D=A
                @3
                D=D+A
                @0
                M=D
            ";
        let mut parser = Parser::new(input);
    }
}
