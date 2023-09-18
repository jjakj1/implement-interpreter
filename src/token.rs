use once_cell::sync::Lazy;
use std::collections::HashMap;

#[derive(Debug, Clone)]
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

static KEYWORDS: Lazy<HashMap<&'static str, TokenType>> = Lazy::new(|| {
    HashMap::from([
        ("fn", TokenType::Function),
        ("let", TokenType::Let),
        ("true", TokenType::True),
        ("false", TokenType::False),
        ("if", TokenType::If),
        ("else", TokenType::Else),
        ("return", TokenType::Return),
    ])
});

pub fn lookup_identifier(identifier: &str) -> TokenType {
    *KEYWORDS.get(identifier).unwrap_or(&TokenType::Ident)
}

#[derive(PartialEq, Eq, Hash, Debug, Clone, Copy)]
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
    String,
    LeftBracket,
    RightBracket,
    Colon,
}
