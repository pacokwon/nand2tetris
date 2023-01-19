use std::fs::File;
use std::io::Write;

use crate::span::Span;
use crate::xml_printer::{print_closing, print_opening, print_tag, XmlPrinter};

#[derive(Debug, Clone)]
pub struct Token {
    pub token_type: TokenType,
    pub span: Span,
}

impl Default for Token {
    fn default() -> Self {
        Token {
            token_type: TokenType::Error,
            span: Span::default(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TokenType {
    Symbol(char),       // a single character representing the symbol
    Identifier(String), // a sequence of letters, digits, underscore not starting with digit
    Keyword(KeywordType),
    Integer(u16),
    String(String),
    Eof,
    Error,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeywordType {
    Class,
    Method,
    Function,
    Constructor,
    IntType,
    BoolType,
    CharType,
    Void,
    Var,
    Static,
    Field,
    Let,
    Do,
    If,
    Else,
    While,
    Return,
    True,
    False,
    Null,
    This,
}

impl XmlPrinter for TokenType {
    fn print_xml(&self, file: &mut File) {
        use TokenType::*;

        match self {
            Symbol(c) => c.print_xml(file),
            Identifier(name) => print_tag(file, "identifier", name),
            Keyword(k) => k.print_xml(file),
            Integer(num) => writeln!(file, "<integerConstant> {num} </integerConstant>").unwrap(),
            String(name) => print_tag(file, "stringConstant", name),
            Eof => (),
            Error => panic!("Not supposed to be printed in xml format"),
        }
    }
}

impl XmlPrinter for KeywordType {
    fn print_xml(&self, file: &mut File) {
        use KeywordType::*;

        match self {
            Class => print_tag(file, "keyword", "class"),
            Method => print_tag(file, "keyword", "method"),
            Function => print_tag(file, "keyword", "function"),
            Constructor => print_tag(file, "keyword", "constructor"),
            IntType => print_tag(file, "keyword", "int"),
            BoolType => print_tag(file, "keyword", "boolean"),
            CharType => print_tag(file, "keyword", "char"),
            Void => print_tag(file, "keyword", "void"),
            Var => print_tag(file, "keyword", "var"),
            Static => print_tag(file, "keyword", "static"),
            Field => print_tag(file, "keyword", "field"),
            Let => print_tag(file, "keyword", "let"),
            Do => print_tag(file, "keyword", "do"),
            If => print_tag(file, "keyword", "if"),
            Else => print_tag(file, "keyword", "else"),
            While => print_tag(file, "keyword", "while"),
            Return => print_tag(file, "keyword", "return"),
            True => print_tag(file, "keyword", "true"),
            False => print_tag(file, "keyword", "false"),
            Null => print_tag(file, "keyword", "null"),
            This => print_tag(file, "keyword", "this"),
        }
    }
}

impl XmlPrinter for Vec<TokenType> {
    fn print_xml(&self, file: &mut File) {
        print_opening(file, "tokens");
        self.iter().for_each(|t| t.print_xml(file));
        print_closing(file, "tokens");
    }
}
