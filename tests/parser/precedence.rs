use crate::parser::helpers::parse_program_from;
use implement_parser::ast::traits::Node;
use rstest::rstest;

#[rstest]
#[case("-a * b".to_owned(), "((-a) * b)".to_owned())]
#[case("!-a".to_owned(), "(!(-a))".to_owned())]
#[case("a + b + c".to_owned(), "((a + b) + c)".to_owned())]
#[case("a + b - c".to_owned(), "((a + b) - c)".to_owned())]
#[case("a * b * c".to_owned(), "((a * b) * c)".to_owned())]
#[case("a * b / c".to_owned(), "((a * b) / c)".to_owned())]
#[case("a + b / c".to_owned(), "(a + (b / c))".to_owned())]
#[case("a + b * c + d / e - f".to_owned(), "(((a + (b * c)) + (d / e)) - f)".to_owned())]
#[case("3 + 4; -5 * 5".to_owned(), "(3 + 4)((-5) * 5)".to_owned())]
#[case("5 > 4 == 3 < 4".to_owned(), "((5 > 4) == (3 < 4))".to_owned())]
#[case("5 < 4 != 3 > 4".to_owned(), "((5 < 4) != (3 > 4))".to_owned())]
#[case("3 + 4 * 5 == 3 * 1 + 4 * 5".to_owned(), "((3 + (4 * 5)) == ((3 * 1) + (4 * 5)))".to_owned())]
#[case("true".to_owned(), "true".to_owned())]
#[case("false".to_owned(), "false".to_owned())]
#[case("3 > 5 == false".to_owned(), "((3 > 5) == false)".to_owned())]
#[case("3 < 5 == true".to_owned(), "((3 < 5) == true)".to_owned())]
#[case("1 + (2 + 3) + 4".to_owned(), "((1 + (2 + 3)) + 4)".to_owned())]
#[case("(5 + 5) * 2".to_owned(), "((5 + 5) * 2)".to_owned())]
#[case("2 / (5 + 5)".to_owned(), "(2 / (5 + 5))".to_owned())]
#[case("-(5 + 5)".to_owned(), "(-(5 + 5))".to_owned())]
#[case("!(true == true)".to_owned(), "(!(true == true))".to_owned())]
#[case("a + add(b * c) + d".to_owned(), "((a + add((b * c))) + d)".to_owned())]
#[case("add(a, b, 1, 2 * 3, 4 + 5, add(6, 7 * 8))".to_owned(), "add(a, b, 1, (2 * 3), (4 + 5), add(6, (7 * 8)))".to_owned())]
#[case("add(a + b + c * d / f + g)".to_owned(), "add((((a + b) + ((c * d) / f)) + g))".to_owned())]
#[case("a * [1, 2, 3, 4][b * c] * d".to_owned(), "((a * ([1, 2, 3, 4][(b * c)])) * d)".to_owned())]
#[case("add(a * b[2], b[1], 2 * [1, 2][1])".to_owned(), "add((a * (b[2])), (b[1]), (2 * ([1, 2][1])))".to_owned())]
fn test_operator_precedence_parsing(#[case] input: String, #[case] expected: String) {
    let program = parse_program_from(input);
    assert_eq!(program.string(), expected);
}
