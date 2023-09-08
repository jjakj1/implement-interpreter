use implement_parser::lexer::Lexer;
use implement_parser::token::TokenType;

#[test]
fn test_simple_input_token() {
    let input = "=+(){},;";

    let tests = [
        (TokenType::Assign, "="),
        (TokenType::Plus, "+"),
        (TokenType::LeftParen, "("),
        (TokenType::RightParen, ")"),
        (TokenType::LeftBrace, "{"),
        (TokenType::RightBrace, "}"),
        (TokenType::Comma, ","),
        (TokenType::Semicolon, ";"),
        (TokenType::EOF, ""),
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
        (TokenType::Let, "let"),
        (TokenType::Ident, "five"),
        (TokenType::Assign, "="),
        (TokenType::Int, "5"),
        (TokenType::Semicolon, ";"),
        (TokenType::Let, "let"),
        (TokenType::Ident, "ten"),
        (TokenType::Assign, "="),
        (TokenType::Int, "10"),
        (TokenType::Semicolon, ";"),
        (TokenType::Let, "let"),
        (TokenType::Ident, "add"),
        (TokenType::Assign, "="),
        (TokenType::Function, "fn"),
        (TokenType::LeftParen, "("),
        (TokenType::Ident, "x"),
        (TokenType::Comma, ","),
        (TokenType::Ident, "y"),
        (TokenType::RightParen, ")"),
        (TokenType::LeftBrace, "{"),
        (TokenType::Ident, "x"),
        (TokenType::Plus, "+"),
        (TokenType::Ident, "y"),
        (TokenType::Semicolon, ";"),
        (TokenType::RightBrace, "}"),
        (TokenType::Let, "let"),
        (TokenType::Ident, "result"),
        (TokenType::Assign, "="),
        (TokenType::Ident, "add"),
        (TokenType::LeftParen, "("),
        (TokenType::Ident, "five"),
        (TokenType::Comma, ","),
        (TokenType::Ident, "ten"),
        (TokenType::RightParen, ")"),
        (TokenType::Semicolon, ";"),
        (TokenType::Bang, "!"),
        (TokenType::Minus, "-"),
        (TokenType::Slash, "/"),
        (TokenType::Asterisk, "*"),
        (TokenType::Int, "5"),
        (TokenType::Semicolon, ";"),
        (TokenType::Int, "5"),
        (TokenType::LessThan, "<"),
        (TokenType::Int, "10"),
        (TokenType::GreaterThan, ">"),
        (TokenType::Int, "5"),
        (TokenType::Semicolon, ";"),
        (TokenType::If, "if"),
        (TokenType::LeftParen, "("),
        (TokenType::Int, "5"),
        (TokenType::LessThan, "<"),
        (TokenType::Int, "10"),
        (TokenType::RightParen, ")"),
        (TokenType::LeftBrace, "{"),
        (TokenType::Return, "return"),
        (TokenType::True, "true"),
        (TokenType::Semicolon, ";"),
        (TokenType::RightBrace, "}"),
        (TokenType::Else, "else"),
        (TokenType::LeftBrace, "{"),
        (TokenType::Return, "return"),
        (TokenType::False, "false"),
        (TokenType::Semicolon, ";"),
        (TokenType::RightBrace, "}"),
        (TokenType::Int, "10"),
        (TokenType::Equal, "=="),
        (TokenType::Int, "10"),
        (TokenType::Semicolon, ";"),
        (TokenType::Int, "10"),
        (TokenType::NotEqual, "!="),
        (TokenType::Int, "9"),
        (TokenType::Semicolon, ";"),
        (TokenType::EOF, ""),
    ];

    let mut lexer = Lexer::new(input.to_owned());
    for test in tests.iter() {
        let token = lexer.next_token();
        assert_eq!(token.token_type, test.0);
        assert_eq!(token.literal, test.1);
    }
}
