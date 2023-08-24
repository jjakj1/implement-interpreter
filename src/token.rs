use std::collections::HashMap;

type TokenType = String;

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
        ("fn", FUNCTION.to_owned()),
        ("let", LET.to_owned()),
        ("true", TRUE.to_owned()),
        ("false", FALSE.to_owned()),
        ("if", IF.to_owned()),
        ("else", ELSE.to_owned()),
        ("return", RETURN.to_owned()),
    ]);

    if let Some(token_type) = keywords.get(identifier) {
        token_type.to_owned()
    } else {
        IDENT.to_owned()
    }
}

pub const ILLEGAL: &str = "ILLEGAL";
pub const EOF: &str = "EOF";

pub const IDENT: &str = "IDENT";
pub const INT: &str = "INT";

pub const ASSIGN: &str = "=";
pub const PLUS: &str = "+";
pub const MINUS: &str = "-";
pub const BANG: &str = "!";
pub const ASTERISK: &str = "*";
pub const SLASH: &str = "/";

pub const LT: &str = "<";
pub const GT: &str = ">";

pub const COMMA: &str = ",";
pub const SEMICOLON: &str = ";";

pub const LPAREN: &str = "(";
pub const RPAREN: &str = ")";
pub const LBARCE: &str = "{";
pub const RBARCE: &str = "}";

pub const EQ: &str = "==";
pub const NOT_EQ: &str = "!=";

// keywords
pub const FUNCTION: &str = "FUNCTION";
pub const LET: &str = "LET";
pub const TRUE: &str = "TRUE";
pub const FALSE: &str = "FALSE";
pub const IF: &str = "IF";
pub const ELSE: &str = "ELSE";
pub const RETURN: &str = "RETURN";
