use crate::parser::helpers::parse_program_from;
use implement_parser::ast::traits::Node;

#[test]
fn test_operator_precedence_parsing() {
    struct PrecedenceTest {
        input: String,
        expect: String,
    }

    let precedent_tests = [
        PrecedenceTest {
            input: "-a * b".to_owned(),
            expect: "((-a) * b)".to_owned(),
        },
        PrecedenceTest {
            input: "!-a".to_owned(),
            expect: "(!(-a))".to_owned(),
        },
        PrecedenceTest {
            input: "a + b + c".to_owned(),
            expect: "((a + b) + c)".to_owned(),
        },
        PrecedenceTest {
            input: "a + b - c".to_owned(),
            expect: "((a + b) - c)".to_owned(),
        },
        PrecedenceTest {
            input: "a * b * c".to_owned(),
            expect: "((a * b) * c)".to_owned(),
        },
        PrecedenceTest {
            input: "a * b / c".to_owned(),
            expect: "((a * b) / c)".to_owned(),
        },
        PrecedenceTest {
            input: "a + b / c".to_owned(),
            expect: "(a + (b / c))".to_owned(),
        },
        PrecedenceTest {
            input: "a + b * c + d / e - f".to_owned(),
            expect: "(((a + (b * c)) + (d / e)) - f)".to_owned(),
        },
        PrecedenceTest {
            input: "3 + 4; -5 * 5".to_owned(),
            expect: "(3 + 4)((-5) * 5)".to_owned(),
        },
        PrecedenceTest {
            input: "5 > 4 == 3 < 4".to_owned(),
            expect: "((5 > 4) == (3 < 4))".to_owned(),
        },
        PrecedenceTest {
            input: "5 < 4 != 3 > 4".to_owned(),
            expect: "((5 < 4) != (3 > 4))".to_owned(),
        },
        PrecedenceTest {
            input: "3 + 4 * 5 == 3 * 1 + 4 * 5".to_owned(),
            expect: "((3 + (4 * 5)) == ((3 * 1) + (4 * 5)))".to_owned(),
        },
        PrecedenceTest {
            input: "true".to_owned(),
            expect: "true".to_owned(),
        },
        PrecedenceTest {
            input: "false".to_owned(),
            expect: "false".to_owned(),
        },
        PrecedenceTest {
            input: "3 > 5 == false".to_owned(),
            expect: "((3 > 5) == false)".to_owned(),
        },
        PrecedenceTest {
            input: "3 < 5 == true".to_owned(),
            expect: "((3 < 5) == true)".to_owned(),
        },
        PrecedenceTest {
            input: "1 + (2 + 3) + 4".to_owned(),
            expect: "((1 + (2 + 3)) + 4)".to_owned(),
        },
        PrecedenceTest {
            input: "(5 + 5) * 2".to_owned(),
            expect: "((5 + 5) * 2)".to_owned(),
        },
        PrecedenceTest {
            input: "2 / (5 + 5)".to_owned(),
            expect: "(2 / (5 + 5))".to_owned(),
        },
        PrecedenceTest {
            input: "-(5 + 5)".to_owned(),
            expect: "(-(5 + 5))".to_owned(),
        },
        PrecedenceTest {
            input: "!(true == true)".to_owned(),
            expect: "(!(true == true))".to_owned(),
        },
        PrecedenceTest {
            input: "a + add(b * c) + d".to_owned(),
            expect: "((a + add((b * c))) + d)".to_owned(),
        },
        PrecedenceTest {
            input: "add(a, b, 1, 2 * 3, 4 + 5, add(6, 7 * 8))".to_owned(),
            expect: "add(a, b, 1, (2 * 3), (4 + 5), add(6, (7 * 8)))".to_owned(),
        },
        PrecedenceTest {
            input: "add(a + b + c * d / f + g)".to_owned(),
            expect: "add((((a + b) + ((c * d) / f)) + g))".to_owned(),
        },
    ];

    for test in precedent_tests {
        let program = parse_program_from(test.input);
        assert_eq!(program.string(), test.expect);
    }
}
