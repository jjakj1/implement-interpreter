use implement_parser::ast::expressions::Identifier;
use implement_parser::ast::program::Program;
use implement_parser::ast::statements::LetStatement;
use implement_parser::ast::traits::{Expression, Node, Statement};
use implement_parser::token::{Token, TokenType};

#[test]
fn test_string() {
    let program = Program {
        statements: vec![Box::new(LetStatement {
            token: Token {
                token_type: TokenType::Let,
                literal: "let".to_owned(),
            },
            name: Identifier {
                token: Token {
                    token_type: TokenType::Ident,
                    literal: "myVar".to_owned(),
                },
                value: "myVar".to_owned(),
            },
            value: Box::new(Identifier {
                token: Token {
                    token_type: TokenType::Ident,
                    literal: "anotherVar".to_owned(),
                },
                value: "anotherVar".to_owned(),
            }) as Box<dyn Expression>,
        }) as Box<dyn Statement>],
    };

    assert_eq!(program.string(), "let myVar = anotherVar;");
}
