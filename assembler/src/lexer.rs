use crate::token::Token;

pub struct Lexer {
    source: Vec<char>,
    position: usize,
    line: usize,
}

impl Lexer {
    pub fn new(source: &str) -> Self {
        let source: Vec<char> = source.to_string().chars().collect();
        let position = 0;
        let line = 0;
        Lexer {
            source,
            position,
            line,
        }
    }

    fn peek(&self) -> char {
        if self.position < self.source.len() {
            self.source[self.position]
        } else {
            '\0'
        }
    }

    fn expect(&mut self, c: char, message: &str) {
        if self.peek() == c {
            self.position += 1;
        } else {
            panic!("{}", message);
        }
    }

    pub fn all_tokens(&mut self) -> Vec<Token> {
        if self.source.len() == 0 {
            return vec![Token::Eof];
        }

        let mut tokens = Vec::new();
        loop {
            match self.token() {
                Token::Eof => {
                    tokens.push(Token::Eof);
                    break tokens;
                }
                token => {
                    tokens.push(token);
                }
            }
        }
    }

    pub fn token(&mut self) -> Token {
        loop {
            let c = self.advance();
            match c {
                '+' => break Token::Plus,
                '-' => break Token::Minus,
                '&' => break Token::Ampersand,
                '@' => break Token::At,
                '|' => break Token::Pipe,
                '=' => break Token::Equal,
                ';' => break Token::Semicolon,
                '!' => break Token::Bang,
                '(' => break Token::LParen,
                ')' => break Token::RParen,
                '\r' | '\n' => {
                    self.line += 1;
                    break Token::Newline;
                }
                ' ' | '\t' => { /* Do nothing */ }
                '/' => {
                    self.expect('/', "Expected two slashes for comment.");
                    while !self.is_at_end() && self.peek() != '\n' {
                        self.position += 1;
                    }

                    if self.peek() == '\n' {
                        self.line += 1;
                        self.position += 1;
                        break Token::Newline;
                    }
                }
                c if c.is_ascii_alphabetic() => {
                    self.position -= 1;
                    break self.symbol();
                }
                c if c.is_ascii_digit() => {
                    self.position -= 1;
                    break self.number();
                }
                _ if self.is_at_end() => break Token::Eof,
                _ => {
                    panic!(
                        "Unexpected character {} encountered at position {}",
                        c,
                        self.position - 1
                    );
                }
            }
        }
    }

    fn advance(&mut self) -> char {
        if self.is_at_end() {
            '\0'
        } else {
            let c = self.peek();
            self.position += 1;
            c
        }
    }

    // NOTE: this function expects that the current character is an alphabet, and not a digit
    //       the caller must guarantee this condition
    fn symbol(&mut self) -> Token {
        let position = self.position;
        while { let peek = self.peek(); peek.is_ascii_alphanumeric() || peek == '_' || peek == '.' || peek == '$' } {
            self.position += 1;
        }

        let symbol = self.source[position..self.position]
            .iter()
            .collect::<String>();

        // handle special symbols, because they have their own variants.
        match &symbol[..] {
            "A" => Token::RegA,
            "M" => Token::RegM,
            "D" => Token::RegD,
            "AM" => Token::RegAM,
            "AD" => Token::RegAD,
            "MD" => Token::RegMD,
            "AMD" => Token::RegAMD,
            "JGT" => Token::JGT,
            "JEQ" => Token::JEQ,
            "JGE" => Token::JGE,
            "JLT" => Token::JLT,
            "JNE" => Token::JNE,
            "JLE" => Token::JLE,
            "JMP" => Token::JMP,
            _ => Token::Symbol(symbol),
        }
    }

    fn number(&mut self) -> Token {
        let position = self.position;
        while self.peek().is_ascii_digit() {
            self.position += 1;
        }

        let num = self.source[position..self.position]
            .iter()
            .collect::<String>()
            .parse::<u16>()
            .expect("Expected number string, but failed to parse into number.");

        Token::Number(num)
    }

    fn is_at_end(&self) -> bool {
        self.position >= self.source.len()
    }
}

#[cfg(test)]
mod tests {
    use super::Lexer;
    use crate::token::Token;

    fn lex_and_get_tokens(input: &str) -> Vec<Token> {
        Lexer::new(input).all_tokens()
    }

    #[test]
    fn test_empty1() {
        use Token::*;
        let input = "";
        let tokens = lex_and_get_tokens(&input);
        let expected = vec![Eof];
        assert_eq!(tokens, expected);
    }

    #[test]
    fn test_empty2() {
        use Token::*;
        let input = "\n  \t";
        let tokens = lex_and_get_tokens(&input);
        let expected = vec![Newline, Eof];
        assert_eq!(tokens, expected);
    }

    #[test]
    fn test_simple_tokens1() {
        use Token::*;
        let input = "+-=();!&@|";
        let tokens = lex_and_get_tokens(&input);
        let expected = vec![
            Plus, Minus, Equal, LParen, RParen, Semicolon, Bang, Ampersand, At, Pipe, Eof,
        ];
        assert_eq!(tokens, expected);
    }

    #[test]
    fn test_simple_tokens2() {
        use Token::*;
        let input = "+-=( ) ;!&@ | ";
        let tokens = lex_and_get_tokens(&input);
        let expected = vec![
            Plus, Minus, Equal, LParen, RParen, Semicolon, Bang, Ampersand, At, Pipe, Eof,
        ];
        assert_eq!(tokens, expected);
    }

    #[test]
    fn test_number() {
        let input = "42";
        let tokens = lex_and_get_tokens(&input);
        let expected = vec![Token::Number(42), Token::Eof];
        assert_eq!(tokens, expected);
    }

    #[test]
    fn test_numbers() {
        use Token::*;

        let input = "42 24";
        let tokens = lex_and_get_tokens(&input);
        let expected = vec![Number(42), Number(24), Eof];
        assert_eq!(tokens, expected);
    }

    #[test]
    fn test_symbol() {
        let input = "foobar";
        let tokens = lex_and_get_tokens(&input);
        let expected = vec![Token::Symbol(String::from("foobar")), Token::Eof];
        assert_eq!(tokens, expected);
    }

    #[test]
    fn test_symbols() {
        use Token::*;

        let input = "foo bar";
        let tokens = lex_and_get_tokens(&input);
        let expected = vec![
            Symbol(String::from("foo")),
            Symbol(String::from("bar")),
            Eof,
        ];
        assert_eq!(tokens, expected);
    }

    #[test]
    fn test_symbol_number() {
        use Token::*;

        let input = "foo 30";
        let tokens = lex_and_get_tokens(&input);
        let expected = vec![Symbol(String::from("foo")), Number(30), Eof];
        assert_eq!(tokens, expected);
    }

    #[test]
    fn test_comment1() {
        use Token::*;

        let input = "// foobar";
        let tokens = lex_and_get_tokens(&input);
        let expected = vec![Eof];
        assert_eq!(tokens, expected);
    }

    #[test]
    fn test_comment2() {
        use Token::*;

        let input = "foobar// foobar";
        let tokens = lex_and_get_tokens(&input);
        let expected = vec![Symbol(String::from("foobar")), Eof];
        assert_eq!(tokens, expected);
    }

    #[test]
    fn test_comment3() {
        use Token::*;

        let input = "foobar// foobar\nfoo";
        let tokens = lex_and_get_tokens(&input);
        let expected = vec![
            Symbol(String::from("foobar")),
            Newline,
            Symbol(String::from("foo")),
            Eof,
        ];
        assert_eq!(tokens, expected);
    }

    #[test]
    fn test_mixed() {
        use Token::*;

        let input = "()&foobar// foobar";
        let tokens = lex_and_get_tokens(&input);
        let expected = vec![
            LParen,
            RParen,
            Ampersand,
            Symbol(String::from("foobar")),
            Eof,
        ];
        assert_eq!(tokens, expected);
    }

    #[test]
    fn test_registers() {
        use Token::*;

        let input = "A M D AM AD MD MDA AMD";
        let tokens = lex_and_get_tokens(&input);
        let expected = vec![
            RegA,
            RegM,
            RegD,
            RegAM,
            RegAD,
            RegMD,
            Symbol(String::from("MDA")),
            RegAMD,
            Eof,
        ];
        assert_eq!(tokens, expected);
    }

    #[test]
    fn test_jumps() {
        use Token::*;

        let input = "JGT JEQ JGE JLT JNE JLE JMP JPM";
        let tokens = lex_and_get_tokens(&input);
        let expected = vec![
            JGT,
            JEQ,
            JGE,
            JLT,
            JNE,
            JLE,
            JMP,
            Symbol(String::from("JPM")),
            Eof,
        ];
        assert_eq!(tokens, expected);
    }
}
