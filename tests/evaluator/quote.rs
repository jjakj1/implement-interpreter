use super::eval::test_eval;
use implement_parser::evaluator::object::Quote;
use rstest::rstest;

#[rstest]
#[case("quote(5)".to_owned(), "5".to_owned())]
#[case("quote(5 + 8)".to_owned(), "(5 + 8)".to_owned())]
#[case("quote(foobar)".to_owned(), "foobar".to_owned())]
#[case("quote(foobar + barfoo)".to_owned(), "(foobar + barfoo)".to_owned())]
fn test_quote(#[case] input: String, #[case] expected: String) {
    let evaluted = test_eval(input);
    let quote = evaluted.downcast_ref::<Quote>().unwrap();
    assert_eq!(quote.node.string(), expected);
}

#[rstest]
#[case("quote(unquote(4))".to_owned(), "4".to_owned())]
#[case("quote(unquote(4 + 4))".to_owned(), "8".to_owned())]
#[case("quote(8 + unquote(4 + 4))".to_owned(), "(8 + 8)".to_owned())]
#[case("quote(unquote(4 + 4) + 8)".to_owned(), "(8 + 8)".to_owned())]
#[case::env(r#"let foobar = 8; quote(foobar);"#.to_owned(), "foobar".to_owned())]
#[case::env(r#"let foobar = 8; quote(unquote(foobar));"#.to_owned(), "8".to_owned())]
#[case::boolean("quote(unquote(true))".to_owned(), "true".to_owned())]
#[case::boolean("quote(unquote(true == false))".to_owned(), "false".to_owned())]
#[case::nested("quote(unquote(quote(4 + 4)))".to_owned(), "(4 + 4)".to_owned())]
#[case::boolean(r#"let quotedInfixExpression = quote(4 + 4); quote(unquote(4 + 4) + unquote(quotedInfixExpression))"#.to_owned(), "(8 + (4 + 4))".to_owned())]
fn test_quote_unquote(#[case] input: String, #[case] expected: String) {
    let evaluated = test_eval(input);
    let quote = evaluated.downcast_ref::<Quote>().unwrap();
    assert_eq!(quote.node.string(), expected);
}
