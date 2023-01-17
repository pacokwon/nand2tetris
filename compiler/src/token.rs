use crate::span::Span;

#[derive(Debug, Clone)]
pub struct Token {
    pub token_type: TokenType,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TokenType {
    Symbol(char), // a single character representing the symbol
    Identifier(String), // a sequence of letters, digits, underscore not starting with digit
    Keyword(KeywordType),
    Integer(u16),
    String(String),
    Eof,
    Error,
}

#[derive(Debug, Clone, PartialEq, Eq)]
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
