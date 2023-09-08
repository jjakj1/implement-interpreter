use crate::token::{self, Token, TokenType};

pub struct Lexer {
    input: String,
    position: usize,
    read_position: usize,
    current_character: Option<char>,
}

impl Lexer {
    pub fn new(input: String) -> Self {
        let mut lexer = Self {
            input,
            position: 0,
            read_position: 0,
            current_character: None,
        };
        lexer.read_character();
        lexer
    }

    pub fn read_character(&mut self) {
        self.current_character = self.input.chars().nth(self.read_position);
        self.position = self.read_position;
        self.read_position += 1;
    }

    pub fn next_token(&mut self) -> Token {
        // TODO: can this be improved? Like removing the bool value
        let mut need_read_next = true;
        self.skip_whitespace();
        // can return value in `match`
        let token =
            self.current_character
                .map_or(Token::new(TokenType::EOF, "".to_owned()), |current| {
                    match current {
                        '=' => {
                            if self.peek_character() == '=' {
                                self.read_character();
                                // token.push(self.current_character);
                                Token::new(
                                    TokenType::Equal,
                                    // `+` on String's right hands is &str
                                    current.to_string()
                                        + &self.current_character.unwrap().to_string(),
                                )
                            } else {
                                Token::new(TokenType::Assign, current.to_string())
                            }
                        }
                        ';' => Token::new(TokenType::Semicolon, current.to_string()),
                        '(' => Token::new(TokenType::LeftParen, current.to_string()),
                        ')' => Token::new(TokenType::RightParen, current.to_string()),
                        ',' => Token::new(TokenType::Comma, current.to_string()),
                        '+' => Token::new(TokenType::Plus, current.to_string()),
                        '-' => Token::new(TokenType::Minus, current.to_string()),
                        '{' => Token::new(TokenType::LeftBrace, current.to_string()),
                        '}' => Token::new(TokenType::RightBrace, current.to_string()),
                        '!' => {
                            if self.peek_character() == '=' {
                                self.read_character();
                                Token::new(
                                    TokenType::NotEqual,
                                    current.to_string()
                                        + &self.current_character.unwrap().to_string(),
                                )
                            } else {
                                Token::new(TokenType::Bang, current.to_string())
                            }
                        }
                        '/' => Token::new(TokenType::Slash, current.to_string()),
                        '*' => Token::new(TokenType::Asterisk, current.to_string()),
                        '<' => Token::new(TokenType::LessThan, current.to_string()),
                        '>' => Token::new(TokenType::GreaterThan, current.to_string()),
                        _ => {
                            if is_letter(current) {
                                let identifier = self.read_identifier();
                                let token_type = token::lookup_identifier(&identifier);
                                need_read_next = false;
                                Token::new(token_type, identifier)
                            } else if current.is_ascii_digit() {
                                need_read_next = false;
                                Token::new(TokenType::Int, self.read_number())
                            } else {
                                Token::new(TokenType::Illegal, current.to_string())
                            }
                        }
                    }
                });

        if need_read_next {
            self.read_character();
        }
        token
    }

    fn read_identifier(&mut self) -> String {
        let start_position = self.position;
        while let Some(current) = self.current_character {
            if is_letter(current) {
                self.read_character()
            } else {
                break;
            }
        }
        // the way to get a substring
        self.input[start_position..self.position].to_owned()
    }

    fn read_number(&mut self) -> String {
        let start_position = self.position;
        while let Some(current) = self.current_character {
            if current.is_ascii_digit() {
                self.read_character();
            } else {
                break;
            }
        }
        self.input[start_position..self.position].to_owned()
    }

    fn skip_whitespace(&mut self) {
        while let Some(current) = self.current_character {
            if is_whitespace(current) {
                self.read_character();
            } else {
                break;
            }
        }
    }

    fn peek_character(&self) -> char {
        self.input
            .chars()
            .nth(self.read_position)
            .unwrap_or_default()
    }
}

fn is_letter(character: char) -> bool {
    character.is_ascii_alphabetic() || character == '_'
}

fn is_whitespace(character: char) -> bool {
    character == ' ' || character == '\t' || character == '\n' || character == '\r'
}
