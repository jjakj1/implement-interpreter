use implement_parser::ast::expressions::{Boolean, Identifier, InfixExpression, IntegerLiteral};
use implement_parser::ast::program::Program;
use implement_parser::ast::statements::ExpressionStatement;
use implement_parser::ast::traits::{Expression, Node};
use implement_parser::lexer::Lexer;
use implement_parser::parser::Parser;

pub fn parse_program_from(input: String) -> Program {
    let lexer = Lexer::new(input);
    let mut parser = Parser::new(lexer);
    let program = parser.parse_program();
    for err in parser.error_messages {
        eprintln!("{}", err);
    }
    program
}

pub fn get_first_expression<T>(program: &Program) -> &T
where
    T: 'static, // TODO: 去掉这个会有问题，好像是类型也需要一个生命周期（https://stackoverflow.com/questions/29740488/parameter-type-may-not-live-long-enough）
{
    program
        .statements
        .first()
        .and_then(|statement| statement.as_any().downcast_ref::<ExpressionStatement>())
        .and_then(|expression_statement| {
            expression_statement.expression.as_any().downcast_ref::<T>()
        })
        .unwrap()
}

pub fn test_integer_literal(expression: &dyn Expression, value: i64) {
    let integer_literal = expression
        .as_any()
        .downcast_ref::<IntegerLiteral>()
        .unwrap();
    assert_eq!(integer_literal.value, value);
}

pub fn test_identifier(expression: &dyn Expression, value: String) {
    let identifier = expression.as_any().downcast_ref::<Identifier>().unwrap();
    assert_eq!(identifier.value, value);
    assert_eq!(identifier.token_literal(), value);
}

pub fn test_boolean_literal(expression: &dyn Expression, value: bool) {
    let boolean_literal = expression.as_any().downcast_ref::<Boolean>().unwrap();
    assert_eq!(boolean_literal.value, value);
}

pub fn test_integer_infix_expression(
    expression: &dyn Expression,
    left: i64,
    operator: &str,
    right: i64,
) {
    let infix_expression = expression
        .as_any()
        .downcast_ref::<InfixExpression>()
        .unwrap();
    test_integer_literal(infix_expression.left.as_ref(), left);
    assert_eq!(infix_expression.operator, operator);
    test_integer_literal(infix_expression.right.as_ref(), right);
}

pub fn test_boolean_infix_expression(
    expression: &dyn Expression,
    left: bool,
    operator: &str,
    right: bool,
) {
    let infix_expression = expression
        .as_any()
        .downcast_ref::<InfixExpression>()
        .unwrap();
    test_boolean_literal(infix_expression.left.as_ref(), left);
    assert_eq!(infix_expression.operator, operator);
    test_boolean_literal(infix_expression.right.as_ref(), right);
}

pub fn test_string_infix_expression(
    expression: &dyn Expression,
    left: &str,
    operator: &str,
    right: &str,
) {
    let infix_expression = expression
        .as_any()
        .downcast_ref::<InfixExpression>()
        .unwrap();
    assert_eq!(infix_expression.left.string(), left);
    assert_eq!(infix_expression.operator, operator);
    assert_eq!(infix_expression.right.string(), right);
}
