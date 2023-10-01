use std::collections::HashMap;

use by_address::ByAddress;
use implement_parser::ast::expressions::{
    ArrayLiteral, FunctionLiteral, HashLiteral, Identifier, IfExpression, IndexExpression,
    InfixExpression, IntegerLiteral, PrefixExpression,
};
use implement_parser::ast::modify::modify;
use implement_parser::ast::program::Program;
use implement_parser::ast::statements::{
    BlockStatement, ExpressionStatement, LetStatement, ReturnStatement,
};
use implement_parser::ast::traits::{Expression, Node, Statement};
use implement_parser::token::{Token, TokenType};
use rstest::rstest;

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

#[rstest]
#[case(&mut one(), &two())]
#[case::program(&mut program(Box::new(one())), &program(Box::new(two())))]
#[case::infix(&mut infix_expression(Box::new(one()), Box::new(two())), &infix_expression(Box::new(two()), Box::new(two())))]
#[case::infix(&mut infix_expression(Box::new(two()), Box::new(two())), &infix_expression(Box::new(two()), Box::new(two())))]
#[case::prefix(&mut prefix_expression(Box::new(one())), &prefix_expression(Box::new(two())))]
#[case::index(&mut index_expression(Box::new(one()), Box::new(one())), &index_expression(Box::new(two()), Box::new(two())))]
#[case::if_exp(&mut if_expression(Box::new(one()), Box::new(one()), Box::new(one())), &if_expression(Box::new(two()), Box::new(two()), Box::new(two())))]
#[case::return_state(&mut return_statement(Box::new(one())), &return_statement(Box::new(two())))]
#[case::let_state(&mut let_statement(Box::new(one())), &let_statement(Box::new(two())))]
#[case::func(&mut function_literal(Box::new(one())), &function_literal(Box::new(two())))]
#[case::array(&mut array_literal(Box::new(one()), Box::new(one())), &array_literal(Box::new(two()), Box::new(two())))]
#[case::hash(&mut hash_literal(Box::new(one()), Box::new(one()), Box::new(one()), Box::new(one())), &hash_literal(Box::new(two()), Box::new(two()), Box::new(two()), Box::new(two())))]
fn test_modify(#[case] input: &mut dyn Node, #[case] expected: &dyn Node) {
    modify(input, &turn_one_into_two);
    assert_eq!(input.string(), expected.string());
}

fn one() -> IntegerLiteral {
    IntegerLiteral {
        token: Token {
            token_type: TokenType::Int,
            literal: "".to_owned(),
        },
        value: 1,
    }
}

fn two() -> IntegerLiteral {
    IntegerLiteral {
        token: Token {
            token_type: TokenType::Int,
            literal: "".to_owned(),
        },
        value: 2,
    }
}

fn turn_one_into_two(node: Box<dyn Node>) -> Box<dyn Node> {
    let cloned = dyn_clone::clone_box(node.as_ref());
    if let Ok(mut integer) = node.downcast::<IntegerLiteral>() {
        if integer.value == 1 {
            integer.value = 2;
            return integer;
        }
    }
    cloned
}

fn program(expression: Box<dyn Expression>) -> Program {
    Program {
        statements: vec![Box::new(ExpressionStatement {
            token: Token {
                token_type: TokenType::Int,
                literal: "".to_owned(),
            },
            expression,
        })],
    }
}

fn infix_expression(left: Box<dyn Expression>, right: Box<dyn Expression>) -> InfixExpression {
    InfixExpression {
        token: Token {
            token_type: TokenType::Plus,
            literal: "+".to_owned(),
        },
        left,
        operator: "+".to_owned(),
        right,
    }
}

fn prefix_expression(right: Box<dyn Expression>) -> PrefixExpression {
    PrefixExpression {
        token: Token {
            token_type: TokenType::Minus,
            literal: "-".to_owned(),
        },
        operator: "-".to_owned(),
        right,
    }
}

fn index_expression(left: Box<dyn Expression>, index: Box<dyn Expression>) -> IndexExpression {
    IndexExpression {
        token: Token {
            token_type: TokenType::LeftBracket,
            literal: "[".to_owned(),
        },
        left,
        index,
    }
}

fn if_expression(
    condition: Box<dyn Expression>,
    consequence: Box<dyn Expression>,
    alernative: Box<dyn Expression>,
) -> IfExpression {
    IfExpression {
        token: Token {
            token_type: TokenType::If,
            literal: "if".to_owned(),
        },
        condition,
        consequence: BlockStatement {
            token: Token {
                token_type: TokenType::LeftBrace,
                literal: "{".to_owned(),
            },
            statements: vec![Box::new(ExpressionStatement {
                token: Token {
                    token_type: TokenType::Int,
                    literal: "".to_owned(),
                },
                expression: consequence,
            })],
        },
        alternative: Some(BlockStatement {
            token: Token {
                token_type: TokenType::LeftBrace,
                literal: "{".to_owned(),
            },
            statements: vec![Box::new(ExpressionStatement {
                token: Token {
                    token_type: TokenType::Int,
                    literal: "".to_owned(),
                },
                expression: alernative,
            })],
        }),
    }
}

fn return_statement(return_value: Box<dyn Expression>) -> ReturnStatement {
    ReturnStatement {
        token: Token {
            token_type: TokenType::Return,
            literal: "return".to_owned(),
        },
        return_value,
    }
}

fn let_statement(value: Box<dyn Expression>) -> LetStatement {
    LetStatement {
        token: Token {
            token_type: TokenType::Let,
            literal: "let".to_owned(),
        },
        name: Identifier {
            token: Token {
                token_type: TokenType::Ident,
                literal: "ident".to_owned(),
            },
            value: "ident".to_owned(),
        },
        value,
    }
}

fn function_literal(expression: Box<dyn Expression>) -> FunctionLiteral {
    FunctionLiteral {
        token: Token {
            token_type: TokenType::Function,
            literal: "fn".to_owned(),
        },
        parameters: vec![],
        body: BlockStatement {
            token: Token {
                token_type: TokenType::LeftBrace,
                literal: "{".to_owned(),
            },
            statements: vec![Box::new(ExpressionStatement {
                token: Token {
                    token_type: TokenType::Int,
                    literal: "".to_owned(),
                },
                expression,
            })],
        },
    }
}

fn array_literal(element1: Box<dyn Expression>, element2: Box<dyn Expression>) -> ArrayLiteral {
    ArrayLiteral {
        token: Token {
            token_type: TokenType::LeftBracket,
            literal: "[".to_owned(),
        },
        elements: vec![element1, element2],
    }
}

fn hash_literal(
    key1: Box<dyn Expression>,
    value1: Box<dyn Expression>,
    key2: Box<dyn Expression>,
    value2: Box<dyn Expression>,
) -> HashLiteral {
    HashLiteral {
        token: Token {
            token_type: TokenType::LeftBracket,
            literal: "[".to_owned(),
        },
        pairs: HashMap::from([(ByAddress(key1), value1), (ByAddress(key2), value2)]),
    }
}
