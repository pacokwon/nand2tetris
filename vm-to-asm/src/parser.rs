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
            .filter(Self::should_include_line)
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
            "label" => CommandType::Label,
            "goto" => CommandType::Goto,
            "if-goto" => CommandType::If,
            "function" => CommandType::Function,
            "call" => CommandType::Call,
            "return" => CommandType::Return,
            c => panic!("Invalid command {} at line {}", c, self.line + 1),
        }
    }

    pub fn arg1(&self) -> &str {
        let line = &self.source[self.line];
        match line[0].as_str() {
            "add" | "sub" | "neg" | "eq" | "gt" | "lt" | "and" | "or" | "not" => line[0].as_str(),
            "pop" | "push" | "label" | "goto" | "if-goto" | "function" | "call" => line[1].as_str(),
            "return" => panic!("Invalid call to arg1 for return operation"),
            c => panic!("Invalid command {} at line {}", c, self.line + 1),
        }
    }

    pub fn arg2(&self) -> u16 {
        let line = &self.source[self.line];
        match line[0].as_str() {
            "add" | "sub" | "neg" | "eq" | "gt" | "lt" | "and" | "or" | "not" | "label"
            | "goto" | "if-goto" => {
                panic!("Invalid call to arg2 for operation {}", line[0])
            }
            "pop" | "push" | "function" | "call" => line[2]
                .parse()
                .expect("Expected integer for second argument."),
            _ => todo!(),
        }
    }

    pub fn should_include_line(line: &Vec<String>) -> bool {
        !(line.is_empty() || line[0].starts_with("//"))
    }
}
