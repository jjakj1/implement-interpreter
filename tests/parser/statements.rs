use crate::parser::helpers;
use implement_parser::ast::statements::{LetStatement, ReturnStatement};
use implement_parser::ast::traits::Node;

use rstest::rstest;

#[rstest]
#[case("let x = 5;".to_owned(), "x".to_owned(), "5".to_owned())]
#[case("let y = true;".to_owned(), "y".to_owned(), "true".to_owned())]
#[case("let foobar = y;".to_owned(), "foobar".to_owned(), "y".to_owned())]
fn test_let_statements(
    #[case] input: String,
    #[case] expected_identifier: String,
    #[case] expected_value: String,
) {
    let program = helpers::parse_program_from(input);
    assert_eq!(program.statements.len(), 1);
    let statement = program
        .statements
        .first()
        .and_then(|statement| statement.downcast_ref::<LetStatement>())
        .unwrap();
    assert_eq!(statement.token_literal(), "let");
    assert_eq!(statement.name.string(), expected_identifier);
    assert_eq!(statement.value.string(), expected_value);
}

#[test]
fn test_return_statements() {
    let input = "
        return 5;
        return 10;
        return 993;"
        .to_owned();

    let program = helpers::parse_program_from(input);
    assert_eq!(program.statements.len(), 3);
    for statment in program.statements.iter() {
        let return_statement = statment
            .downcast_ref::<ReturnStatement>()
            .expect("statement is not `ReturnStatment` type");
        assert_eq!(return_statement.token_literal(), "return")
    }
}
