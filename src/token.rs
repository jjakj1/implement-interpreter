use std::collections::HashMap;

#[derive(Debug)]
pub struct Token {
    pub token_type: TokenType,
    pub literal: String,
}

impl Token {
    pub fn new(token_type: TokenType, literal: String) -> Self {
        Self {
            token_type,
            literal,
        }
    }
}

pub fn lookup_identifier(identifier: &str) -> TokenType {
    let keywords: HashMap<&str, TokenType> = HashMap::from([
        ("fn", TokenType::Function),
        ("let", TokenType::Let),
        ("true", TokenType::True),
        ("false", TokenType::False),
        ("if", TokenType::If),
        ("else", TokenType::Else),
        ("return", TokenType::Return),
    ]);

    *keywords.get(identifier).unwrap_or(&TokenType::Ident)
}

#[derive(PartialEq, Debug, Clone, Copy)]
pub enum TokenType {
    Illegal,
    EOF,
    Ident,
    Int,
    Assign,
    Plus,
    Minus,
    Bang,
    Asterisk,
    Slash,
    LessThan,
    GreaterThan,
    Comma,
    Semicolon,
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    Equal,
    NotEqual,
    Function,
    Let,
    True,
    False,
    If,
    Else,
    Return,
}
