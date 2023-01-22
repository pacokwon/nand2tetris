use crate::{
    span::Span,
    token::{KeywordType, Token, TokenType},
};

pub struct Lexer {
    source: Vec<char>,
    start: usize,
    position: usize,
    line: usize,
    pub current_token: Token,
    pub next_token: Token,
}

impl Lexer {
    pub fn new(source: &str) -> Self {
        let source: Vec<char> = source.to_string().chars().collect();
        let start = 0;
        let position = 0;
        let line = 0;
        let current_token = Token {
            token_type: TokenType::Error,
            span: Span::default(),
        };

        let next_token = Token {
            token_type: TokenType::Error,
            span: Span::default(),
        };

        Lexer {
            source,
            start,
            position,
            line,
            current_token,
            next_token,
        }
    }

    pub fn has_more_tokens(&self) -> bool {
        self.position < self.source.len()
    }

    pub fn get_current_token_type(&self) -> &TokenType {
        &self.current_token.token_type
    }

    pub fn get_next_token_type(&self) -> &TokenType {
        &self.next_token.token_type
    }

    fn peek(&self) -> char {
        if self.position < self.source.len() {
            self.source[self.position]
        } else {
            '\0'
        }
    }

    pub fn all_tokens(&mut self) -> Vec<Token> {
        if self.source.len() == 0 {
            return vec![Token {
                token_type: TokenType::Eof,
                span: Span::default(),
            }];
        }

        let mut tokens = Vec::new();
        self.advance_token();

        loop {
            self.advance_token();
            match &self.current_token {
                t @ Token {
                    token_type: TokenType::Eof,
                    ..
                } => {
                    tokens.push(t.clone());
                    break tokens;
                }
                token => {
                    tokens.push(token.clone());
                }
            }
        }
    }

    pub fn advance_token(&mut self) -> Token {
        if let TokenType::Eof = self.next_token.token_type {
            let old_token = std::mem::replace(&mut self.current_token, Token {
                token_type: TokenType::Eof,
                span: Span::default(),
            });

            return old_token;
        }

        let token = loop {
            self.start = self.position;
            let c = self.advance();
            match c {
                '{' | '}' | '(' | ')' | '[' | ']' | '.' | ',' | ';' | '+' | '-' | '*' | '&'
                | '|' | '<' | '>' | '=' | '~' => {
                    break Token {
                        token_type: TokenType::Symbol(c),
                        span: Span(self.start, self.position),
                    }
                }
                '\r' | '\n' | ' ' | '\t' => { /* Do nothing */ }
                '/' => {
                    let peek = self.peek();

                    if peek == '/' {
                        // it's a comment.
                        while !self.is_at_end() && self.peek() != '\n' {
                            self.advance();
                        }

                        if self.is_at_end() {
                            break Token {
                                token_type: TokenType::Eof,
                                span: Span(0, 0),
                            };
                        }

                        self.advance();
                    } else if peek == '*' {
                        self.position += 1;

                        let result = loop {
                            // increment until we see an asterisk again.
                            while !self.is_at_end() && self.peek() != '*' {
                                self.advance();
                            }

                            if self.is_at_end() {
                                break Some(Token {
                                    token_type: TokenType::Eof,
                                    span: Span(0, 0),
                                });
                            }

                            self.advance();
                            if self.is_at_end() {
                                break Some(Token {
                                    token_type: TokenType::Eof,
                                    span: Span(0, 0),
                                });
                            }

                            if self.peek() == '/' {
                                self.advance();
                                break None;
                            }
                        };

                        match result {
                            Some(tok) => break tok,
                            None => (),
                        }
                    } else {
                        break Token {
                            token_type: TokenType::Symbol('/'),
                            span: Span(self.start, self.position),
                        };
                    }
                }
                '"' => {
                    while !self.is_at_end() {
                        let peek = self.peek();

                        if peek == '\n' || peek == '\r' {
                            panic!("Newline character encountered while parsing string literal at line {}, position {}", self.line, self.position);
                        }

                        if peek == '"' {
                            break;
                        }

                        self.advance();
                    }

                    if self.is_at_end() {
                        break Token {
                            token_type: TokenType::Eof,
                            span: Span(0, 0),
                        };
                    }

                    // skip double quotes.
                    self.advance();

                    let literal = self.source[(self.start + 1)..(self.position - 1)]
                        .iter()
                        .collect::<String>();
                    break Token {
                        token_type: TokenType::String(literal),
                        span: Span(self.start, self.position),
                    };
                }
                c if c.is_ascii_alphabetic() => {
                    self.position -= 1;
                    break self.symbol();
                }
                c if c.is_ascii_digit() => {
                    self.position -= 1;
                    break self.number();
                }
                _ if self.is_at_end() => {
                    break Token {
                        token_type: TokenType::Eof,
                        span: Span(0, 0),
                    }
                }
                _ => {
                    panic!(
                        "Unexpected character {} encountered at position {}",
                        c,
                        self.position - 1
                    );
                }
            }
        };
        std::mem::swap(&mut self.current_token, &mut self.next_token);
        let old_token = std::mem::replace(&mut self.next_token, token);
        old_token
    }

    fn advance(&mut self) -> char {
        if self.is_at_end() {
            '\0'
        } else {
            let c = self.peek();
            self.position += 1;
            if c == '\n' || c == '\r' {
                self.line += 1;
            }
            c
        }
    }

    // NOTE: this function expects that the current character is an alphabet, and not a digit
    //       the caller must guarantee this condition
    fn symbol(&mut self) -> Token {
        let position = self.position;
        while {
            let peek = self.peek();
            peek.is_ascii_alphanumeric() || peek == '_'
        } {
            self.position += 1;
        }

        let symbol = self.source[position..self.position]
            .iter()
            .collect::<String>();

        match Self::try_keyword_type(&symbol) {
            Some(kt) => Token {
                token_type: TokenType::Keyword(kt),
                span: Span(position, self.position),
            },
            None => Token {
                token_type: TokenType::Identifier(symbol),
                span: Span(position, self.position),
            },
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

        Token {
            token_type: TokenType::Integer(num),
            span: Span(position, self.position),
        }
    }

    fn is_at_end(&self) -> bool {
        self.position >= self.source.len()
    }

    fn try_keyword_type(symbol: &str) -> Option<KeywordType> {
        use KeywordType::*;

        match symbol {
            "class" => Some(Class),
            "constructor" => Some(Constructor),
            "function" => Some(Function),
            "method" => Some(Method),
            "field" => Some(Field),
            "static" => Some(Static),
            "var" => Some(Var),
            "int" => Some(IntType),
            "char" => Some(CharType),
            "boolean" => Some(BoolType),
            "void" => Some(Void),
            "true" => Some(True),
            "false" => Some(False),
            "null" => Some(Null),
            "this" => Some(This),
            "let" => Some(Let),
            "do" => Some(Do),
            "if" => Some(If),
            "else" => Some(Else),
            "while" => Some(While),
            "return" => Some(Return),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Lexer;
    use crate::token::KeywordType;
    use crate::token::Token;
    use crate::token::TokenType;

    fn lex_and_get_tokens(input: &str) -> Vec<Token> {
        Lexer::new(input).all_tokens()
    }

    fn lex_and_get_token_types(input: &str) -> Vec<TokenType> {
        Lexer::new(input)
            .all_tokens()
            .into_iter()
            .map(|t| t.token_type)
            .collect::<Vec<TokenType>>()
    }

    #[test]
    fn test_empty1() {
        use TokenType::*;

        let input = "";
        let tokens = lex_and_get_token_types(&input);
        let expected = vec![Eof];
        assert_eq!(tokens, expected);
    }

    #[test]
    fn test_empty2() {
        use TokenType::*;
        let input = "\n  \t";
        let tokens = lex_and_get_token_types(&input);
        let expected = vec![Eof];
        assert_eq!(tokens, expected);
    }

    #[test]
    fn test_simple_tokens1() {
        use TokenType::*;
        let input = "{}()[].,;+-*&|<>=~";
        let tokens = lex_and_get_token_types(&input);

        let expected = vec![
            Symbol('{'),
            Symbol('}'),
            Symbol('('),
            Symbol(')'),
            Symbol('['),
            Symbol(']'),
            Symbol('.'),
            Symbol(','),
            Symbol(';'),
            Symbol('+'),
            Symbol('-'),
            Symbol('*'),
            Symbol('&'),
            Symbol('|'),
            Symbol('<'),
            Symbol('>'),
            Symbol('='),
            Symbol('~'),
            Eof,
        ];
        assert_eq!(tokens, expected);
    }

    #[test]
    fn test_simple_tokens2() {
        use TokenType::*;
        let input = "{      }( )      [].,   ; +-*&|<>=   ~";
        let tokens = lex_and_get_token_types(&input);
        let expected = vec![
            Symbol('{'),
            Symbol('}'),
            Symbol('('),
            Symbol(')'),
            Symbol('['),
            Symbol(']'),
            Symbol('.'),
            Symbol(','),
            Symbol(';'),
            Symbol('+'),
            Symbol('-'),
            Symbol('*'),
            Symbol('&'),
            Symbol('|'),
            Symbol('<'),
            Symbol('>'),
            Symbol('='),
            Symbol('~'),
            Eof,
        ];
        assert_eq!(tokens, expected);
    }

    #[test]
    fn test_number() {
        let input = "42";
        let tokens = lex_and_get_token_types(&input);
        let expected = vec![TokenType::Integer(42), TokenType::Eof];
        assert_eq!(tokens, expected);
    }

    #[test]
    fn test_numbers() {
        use TokenType::*;

        let input = "42 24";
        let tokens = lex_and_get_token_types(&input);
        let expected = vec![Integer(42), Integer(24), Eof];
        assert_eq!(tokens, expected);
    }

    #[test]
    fn test_keyword() {
        use KeywordType::*;
        let input = "class constructor function method field static var int char boolean void true false null this let do if else while return";
        let tokens = lex_and_get_token_types(&input);

        let expected = vec![
            TokenType::Keyword(Class),
            TokenType::Keyword(Constructor),
            TokenType::Keyword(Function),
            TokenType::Keyword(Method),
            TokenType::Keyword(Field),
            TokenType::Keyword(Static),
            TokenType::Keyword(Var),
            TokenType::Keyword(IntType),
            TokenType::Keyword(CharType),
            TokenType::Keyword(BoolType),
            TokenType::Keyword(Void),
            TokenType::Keyword(True),
            TokenType::Keyword(False),
            TokenType::Keyword(Null),
            TokenType::Keyword(This),
            TokenType::Keyword(Let),
            TokenType::Keyword(Do),
            TokenType::Keyword(If),
            TokenType::Keyword(Else),
            TokenType::Keyword(While),
            TokenType::Keyword(Return),
            TokenType::Eof,
        ];
        assert_eq!(tokens, expected);
    }

    #[test]
    fn test_identifiers() {
        use KeywordType::*;
        use TokenType::*;
        let input = "foo bar class constructor function method field static var int char boolean void true false null this let do if else while return";
        let tokens = lex_and_get_token_types(&input);

        let expected = vec![
            Identifier(std::string::String::from("foo")),
            Identifier(std::string::String::from("bar")),
            Keyword(Class),
            Keyword(Constructor),
            Keyword(Function),
            Keyword(Method),
            Keyword(Field),
            Keyword(Static),
            Keyword(Var),
            Keyword(IntType),
            Keyword(CharType),
            Keyword(BoolType),
            Keyword(Void),
            Keyword(True),
            Keyword(False),
            Keyword(Null),
            Keyword(This),
            Keyword(Let),
            Keyword(Do),
            Keyword(If),
            Keyword(Else),
            Keyword(While),
            Keyword(Return),
            Eof,
        ];
        assert_eq!(tokens, expected);
    }

    #[test]
    fn test_symbol_number() {
        use TokenType::*;

        let input = "foo 30";
        let tokens = lex_and_get_token_types(&input);
        let expected = vec![
            Identifier(std::string::String::from("foo")),
            Integer(30),
            Eof,
        ];
        assert_eq!(tokens, expected);
    }

    #[test]
    fn test_comment1() {
        use TokenType::*;

        let input = "// foobar";
        let tokens = lex_and_get_token_types(&input);
        let expected = vec![Eof];
        assert_eq!(tokens, expected);
    }

    #[test]
    fn test_comment2() {
        use TokenType::*;

        let input = "foobar// foobar";
        let tokens = lex_and_get_token_types(&input);
        let expected = vec![Identifier(std::string::String::from("foobar")), Eof];
        assert_eq!(tokens, expected);
    }

    #[test]
    fn test_comment3() {
        use TokenType::*;

        let input = "foobar// foobar\nfoo";
        let tokens = lex_and_get_token_types(&input);
        let expected = vec![
            Identifier(std::string::String::from("foobar")),
            Identifier(std::string::String::from("foo")),
            Eof,
        ];
        assert_eq!(tokens, expected);
    }

    #[test]
    fn test_comment4() {
        use TokenType::*;

        let input = "foobar/* foobar */";
        let tokens = lex_and_get_token_types(&input);
        let expected = vec![Identifier(std::string::String::from("foobar")), Eof];
        assert_eq!(tokens, expected);
    }

    #[test]
    fn test_comment5() {
        use TokenType::*;

        let input = "foobar/* foobar */ baz";
        let tokens = lex_and_get_token_types(&input);
        let expected = vec![
            Identifier(std::string::String::from("foobar")),
            Identifier(std::string::String::from("baz")),
            Eof,
        ];
        assert_eq!(tokens, expected);
    }

    #[test]
    fn test_comment6() {
        use TokenType::*;

        let input = "foobar/* foobar *** baz */ baz";
        let tokens = lex_and_get_token_types(&input);
        let expected = vec![
            Identifier(std::string::String::from("foobar")),
            Identifier(std::string::String::from("baz")),
            Eof,
        ];
        assert_eq!(tokens, expected);
    }

    #[test]
    fn test_comment7() {
        use TokenType::*;

        let input = "
            foobar/** foobar ***
                     baz
            */ baz";
        let tokens = lex_and_get_token_types(&input);
        let expected = vec![
            Identifier(std::string::String::from("foobar")),
            Identifier(std::string::String::from("baz")),
            Eof,
        ];
        assert_eq!(tokens, expected);
    }

    #[test]
    fn test_mixed() {
        use TokenType::*;

        let input = "()&foobar// foobar";
        let tokens = lex_and_get_token_types(&input);
        let expected = vec![
            Symbol('('),
            Symbol(')'),
            Symbol('&'),
            Identifier(std::string::String::from("foobar")),
            Eof,
        ];
        assert_eq!(tokens, expected);
    }

    #[test]
    fn test_string1() {
        use TokenType::*;

        let input = "\"foobar\"";
        let tokens = lex_and_get_token_types(&input);
        let expected = vec![String(std::string::String::from("foobar")), Eof];
        assert_eq!(tokens, expected);
    }

    #[test]
    fn test_string2() {
        use TokenType::*;

        let input = "\"hello world\" \"this is a string literal\"";
        let tokens = lex_and_get_token_types(&input);
        let expected = vec![
            String(std::string::String::from("hello world")),
            String(std::string::String::from("this is a string literal")),
            Eof,
        ];
        assert_eq!(tokens, expected);
    }

    #[test]
    fn test_string3() {
        use TokenType::*;

        let input = "\"hello world\" 30 \"this is a string literal\"";
        let tokens = lex_and_get_token_types(&input);
        let expected = vec![
            String(std::string::String::from("hello world")),
            Integer(30),
            String(std::string::String::from("this is a string literal")),
            Eof,
        ];
        assert_eq!(tokens, expected);
    }

    #[test]
    fn test_dot_identifier() {
        use TokenType::*;

        let input = "foo.bar";
        let tokens = lex_and_get_token_types(&input);
        println!("{:?}", tokens);
        let expected = vec![
            Identifier(std::string::String::from("foo")),
            Symbol('.'),
            Identifier(std::string::String::from("bar")),
            Eof,
        ];
        assert_eq!(tokens, expected);
    }
}
