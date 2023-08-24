use crate::token::{self, Token};

pub struct Lexer {
    input: String,
    position: usize,
    read_position: usize,
    current_character: char,
}

impl Lexer {
    pub fn new(input: String) -> Self {
        let mut lexer = Self {
            input,
            position: 0,
            read_position: 0,
            current_character: char::default(),
        };
        lexer.read_character();
        lexer
    }

    pub fn read_character(&mut self) {
        if let Some(next) = self.input.chars().nth(self.read_position) {
            self.current_character = next;
        } else {
            self.current_character = char::default();
        }
        self.position = self.read_position;
        self.read_position += 1;
    }

    pub fn next_token(&mut self) -> token::Token {
        // TODO: can this be improved? Like removing the bool value
        let mut need_read_next = true;
        self.skip_whitespace();
        // can return value in `match`
        let token = match self.current_character {
            // when convert a &str to String, using `to_owned`, which's performance is better
            // https://medium.com/@ericdreichert/converting-str-to-string-vs-to-owned-with-two-benchmarks-a66fd5a081ce#:~:text=to_string()%20is%20the%20generic,the%20literal%20into%20the%20buffer.
            '=' => {
                if self.peek_character() == '=' {
                    let first_char = self.current_character;
                    self.read_character();
                    // token.push(self.current_character);
                    Token::new(
                        token::EQ.to_owned(),
                        // `+` on String's right hands is &str
                        first_char.to_string() + &self.current_character.to_string(),
                    )
                } else {
                    Token::new(token::ASSIGN.to_owned(), self.current_character.to_string())
                }
            }
            ';' => Token::new(
                token::SEMICOLON.to_owned(),
                self.current_character.to_string(),
            ),
            '(' => Token::new(token::LPAREN.to_owned(), self.current_character.to_string()),
            ')' => Token::new(token::RPAREN.to_owned(), self.current_character.to_string()),
            ',' => Token::new(token::COMMA.to_owned(), self.current_character.to_string()),
            '+' => Token::new(token::PLUS.to_owned(), self.current_character.to_string()),
            '{' => Token::new(token::LBARCE.to_owned(), self.current_character.to_string()),
            '}' => Token::new(token::RBARCE.to_owned(), self.current_character.to_string()),
            '-' => Token::new(token::MINUS.to_owned(), self.current_character.to_string()),
            '!' => {
                if self.peek_character() == '=' {
                    let mut token = self.current_character.to_string();
                    self.read_character();
                    token.push(self.current_character);
                    Token::new(token::NOT_EQ.to_owned(), token)
                } else {
                    Token::new(token::BANG.to_owned(), self.current_character.to_string())
                }
            }
            '/' => Token::new(token::SLASH.to_owned(), self.current_character.to_string()),
            '*' => Token::new(
                token::ASTERISK.to_owned(),
                self.current_character.to_string(),
            ),
            '<' => Token::new(token::LT.to_owned(), self.current_character.to_string()),
            '>' => Token::new(token::GT.to_owned(), self.current_character.to_string()),
            '\0' => Token::new(token::EOF.to_owned(), char::default().to_string()), // TODO: 可能有点问题
            _ => {
                if is_letter(self.current_character) {
                    let identifier = self.read_identifier();
                    let token_type = token::lookup_identifier(&identifier);
                    need_read_next = false;
                    Token::new(token_type, identifier)
                } else if self.current_character.is_ascii_digit() {
                    need_read_next = false;
                    Token::new(token::INT.to_owned(), self.read_number())
                } else {
                    Token::new(
                        token::ILLEGAL.to_owned(),
                        self.current_character.to_string(),
                    )
                }
            }
        };
        if need_read_next {
            self.read_character();
        }
        token
    }

    fn read_identifier(&mut self) -> String {
        let start_position = self.position;
        while is_letter(self.current_character) {
            self.read_character();
        }
        // the way to get a substring
        self.input[start_position..self.position].to_owned()
    }

    fn read_number(&mut self) -> String {
        let start_position = self.position;
        while self.current_character.is_ascii_digit() {
            self.read_character();
        }
        self.input[start_position..self.position].to_owned()
    }

    fn skip_whitespace(&mut self) {
        while is_whitespace(self.current_character) {
            self.read_character()
        }
    }

    fn peek_character(&self) -> char {
        if let Some(next) = self.input.chars().nth(self.read_position) {
            next
        } else {
            char::default()
        }
    }
}

fn is_letter(character: char) -> bool {
    character.is_ascii_alphabetic() || character == '_'
}

fn is_whitespace(character: char) -> bool {
    character == ' ' || character == '\t' || character == '\n' || character == '\r'
}

#[cfg(test)]
mod tests {
    use super::Lexer;
    use crate::token;

    #[test]
    fn test_simple_input_token() {
        let input = "=+(){},;";

        let tests = [
            (token::ASSIGN, "="),
            (token::PLUS, "+"),
            (token::LPAREN, "("),
            (token::RPAREN, ")"),
            (token::LBARCE, "{"),
            (token::RBARCE, "}"),
            (token::COMMA, ","),
            (token::SEMICOLON, ";"),
            (token::EOF, "\0"),
        ];

        let mut lexer = Lexer::new(input.to_owned());
        for test in tests.iter() {
            let token = lexer.next_token();
            assert_eq!(token.token_type, test.0);
            assert_eq!(token.literal, test.1);
        }
    }

    #[test]
    fn test_source_code_token() {
        let input = "let five = 5;
            let ten = 10;
            let add = fn(x, y) {
                x + y;
            }

            let result = add(five, ten);

            !-/*5;
            5 < 10 > 5;

            if (5 < 10) {
                return true;
            } else {
                return false;
            }

            10 == 10;
            10 != 9;";

        let tests = [
            (token::LET, "let"),
            (token::IDENT, "five"),
            (token::ASSIGN, "="),
            (token::INT, "5"),
            (token::SEMICOLON, ";"),
            (token::LET, "let"),
            (token::IDENT, "ten"),
            (token::ASSIGN, "="),
            (token::INT, "10"),
            (token::SEMICOLON, ";"),
            (token::LET, "let"),
            (token::IDENT, "add"),
            (token::ASSIGN, "="),
            (token::FUNCTION, "fn"),
            (token::LPAREN, "("),
            (token::IDENT, "x"),
            (token::COMMA, ","),
            (token::IDENT, "y"),
            (token::RPAREN, ")"),
            (token::LBARCE, "{"),
            (token::IDENT, "x"),
            (token::PLUS, "+"),
            (token::IDENT, "y"),
            (token::SEMICOLON, ";"),
            (token::RBARCE, "}"),
            (token::LET, "let"),
            (token::IDENT, "result"),
            (token::ASSIGN, "="),
            (token::IDENT, "add"),
            (token::LPAREN, "("),
            (token::IDENT, "five"),
            (token::COMMA, ","),
            (token::IDENT, "ten"),
            (token::RPAREN, ")"),
            (token::SEMICOLON, ";"),
            (token::BANG, "!"),
            (token::MINUS, "-"),
            (token::SLASH, "/"),
            (token::ASTERISK, "*"),
            (token::INT, "5"),
            (token::SEMICOLON, ";"),
            (token::INT, "5"),
            (token::LT, "<"),
            (token::INT, "10"),
            (token::GT, ">"),
            (token::INT, "5"),
            (token::SEMICOLON, ";"),
            (token::IF, "if"),
            (token::LPAREN, "("),
            (token::INT, "5"),
            (token::LT, "<"),
            (token::INT, "10"),
            (token::RPAREN, ")"),
            (token::LBARCE, "{"),
            (token::RETURN, "return"),
            (token::TRUE, "true"),
            (token::SEMICOLON, ";"),
            (token::RBARCE, "}"),
            (token::ELSE, "else"),
            (token::LBARCE, "{"),
            (token::RETURN, "return"),
            (token::FALSE, "false"),
            (token::SEMICOLON, ";"),
            (token::RBARCE, "}"),
            (token::INT, "10"),
            (token::EQ, "=="),
            (token::INT, "10"),
            (token::SEMICOLON, ";"),
            (token::INT, "10"),
            (token::NOT_EQ, "!="),
            (token::INT, "9"),
            (token::SEMICOLON, ";"),
            (token::EOF, "\0"),
        ];

        let mut lexer = Lexer::new(input.to_owned());
        for test in tests.iter() {
            let token = lexer.next_token();
            assert_eq!(token.token_type, test.0);
            assert_eq!(token.literal, test.1);
        }
    }
}
