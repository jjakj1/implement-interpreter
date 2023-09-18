use std::collections::HashMap;

use crate::parser::helpers::{
    get_first_expression, parse_program_from, test_boolean_infix_expression, test_boolean_literal,
    test_identifier, test_integer_infix_expression, test_integer_literal,
    test_string_infix_expression,
};
use implement_parser::ast::expressions::{
    ArrayLiteral, Boolean, CallExpression, FunctionLiteral, HashLiteral, Identifier, IfExpression,
    IndexExpression, InfixExpression, IntegerLiteral, PrefixExpression, StringLiteral,
};
use implement_parser::ast::program::Program;
use implement_parser::ast::statements::ExpressionStatement;
use implement_parser::ast::traits::{Expression, Node};

use rstest::rstest;

#[test]
fn test_indentifier_expression() {
    let input = "foobar".to_owned();

    let program = parse_program_from(input);
    assert_eq!(program.statements.len(), 1);

    let expression = get_first_expression::<Identifier>(&program);

    assert_eq!(expression.value, "foobar");
    assert_eq!(expression.token_literal(), "foobar");
}

#[test]
fn test_integer_literal_expression() {
    let input = "5;".to_owned();
    let program = parse_program_from(input);
    assert_eq!(program.statements.len(), 1);

    let integer_literal = get_first_expression::<IntegerLiteral>(&program);

    assert_eq!(integer_literal.value, 5);
    assert_eq!(integer_literal.token_literal(), "5");
}

#[rstest]
#[case("true".to_owned(), true)]
#[case("false".to_owned(), false)]
fn test_boolean_expression(#[case] input: String, #[case] expected: bool) {
    let program = parse_program_from(input);
    assert_eq!(program.statements.len(), 1);

    let bool_expression = get_first_expression::<Boolean>(&program);
    assert_eq!(bool_expression.value, expected);
}

#[test]
fn test_prefix_expression() {
    trait PrefixTest {
        fn input(&self) -> String;
        fn test_expression(&self, program: &Program);
    }

    struct IntegerPrefixTest {
        input: String,
        operator: String,
        integer_value: i64,
    }

    impl PrefixTest for IntegerPrefixTest {
        fn input(&self) -> String {
            self.input.clone()
        }

        fn test_expression(&self, program: &Program) {
            let expression = get_first_expression::<PrefixExpression>(program);
            assert_eq!(expression.operator, self.operator);
            test_integer_literal(expression.right.as_ref(), self.integer_value);
        }
    }

    struct BooleanPrefixTest {
        input: String,
        operator: String,
        boolean_value: bool,
    }

    impl PrefixTest for BooleanPrefixTest {
        fn input(&self) -> String {
            self.input.clone()
        }

        fn test_expression(&self, program: &Program) {
            let expression = get_first_expression::<PrefixExpression>(program);
            assert_eq!(expression.operator, self.operator);
            test_boolean_literal(expression.right.as_ref(), self.boolean_value);
        }
    }

    let tests: Vec<Box<dyn PrefixTest>> = vec![
        Box::new(IntegerPrefixTest {
            input: "!5".to_owned(),
            operator: "!".to_owned(),
            integer_value: 5,
        }),
        Box::new(IntegerPrefixTest {
            input: "-15".to_owned(),
            operator: "-".to_owned(),
            integer_value: 15,
        }),
        Box::new(BooleanPrefixTest {
            input: "!true".to_owned(),
            operator: "!".to_owned(),
            boolean_value: true,
        }),
        Box::new(BooleanPrefixTest {
            input: "!false".to_owned(),
            operator: "!".to_owned(),
            boolean_value: false,
        }),
    ];

    for test in tests {
        let program = parse_program_from(test.input());
        assert_eq!(program.statements.len(), 1);

        test.test_expression(&program);
    }
}

#[test]
fn test_parsing_infix_expression() {
    trait InfixTest {
        fn input(&self) -> String;
        fn test_expression(&self, program: &Program);
    }

    // Go 里面直接把 left_value 的类型定义为了 interface {}，Rust 好像里面没有等价的表示任意类型的东西
    struct IntegerInfixTest {
        input: String,
        left_value: i64,
        operator: String,
        right_value: i64,
    }

    impl InfixTest for IntegerInfixTest {
        fn input(&self) -> String {
            self.input.clone()
        }

        fn test_expression(&self, program: &Program) {
            let expression = get_first_expression::<InfixExpression>(program);
            test_integer_infix_expression(
                expression,
                self.left_value,
                &self.operator,
                self.right_value,
            );
        }
    }

    struct BooleanInfixTest {
        input: String,
        left_value: bool,
        operator: String,
        right_value: bool,
    }

    impl InfixTest for BooleanInfixTest {
        fn input(&self) -> String {
            self.input.clone()
        }

        fn test_expression(&self, program: &Program) {
            let expression = get_first_expression::<InfixExpression>(program);
            test_boolean_infix_expression(
                expression,
                self.left_value,
                &self.operator,
                self.right_value,
            )
        }
    }

    // 必须要用 Box 分配在 Heap 上
    let tests: Vec<Box<dyn InfixTest>> = vec![
        Box::new(IntegerInfixTest {
            input: "5 + 5;".to_owned(),
            left_value: 5,
            operator: "+".to_owned(),
            right_value: 5,
        }),
        Box::new(IntegerInfixTest {
            input: "5 - 5;".to_owned(),
            left_value: 5,
            operator: "-".to_owned(),
            right_value: 5,
        }),
        Box::new(IntegerInfixTest {
            input: "5 * 5;".to_owned(),
            left_value: 5,
            operator: "*".to_owned(),
            right_value: 5,
        }),
        Box::new(IntegerInfixTest {
            input: "5 / 5;".to_owned(),
            left_value: 5,
            operator: "/".to_owned(),
            right_value: 5,
        }),
        Box::new(IntegerInfixTest {
            input: "5 > 5;".to_owned(),
            left_value: 5,
            operator: ">".to_owned(),
            right_value: 5,
        }),
        Box::new(IntegerInfixTest {
            input: "5 < 5;".to_owned(),
            left_value: 5,
            operator: "<".to_owned(),
            right_value: 5,
        }),
        Box::new(IntegerInfixTest {
            input: "5 == 5;".to_owned(),
            left_value: 5,
            operator: "==".to_owned(),
            right_value: 5,
        }),
        Box::new(IntegerInfixTest {
            input: "5 != 5;".to_owned(),
            left_value: 5,
            operator: "!=".to_owned(),
            right_value: 5,
        }),
        Box::new(BooleanInfixTest {
            input: "true == true;".to_owned(),
            left_value: true,
            operator: "==".to_owned(),
            right_value: true,
        }),
        Box::new(BooleanInfixTest {
            input: "true != false;".to_owned(),
            left_value: true,
            operator: "!=".to_owned(),
            right_value: false,
        }),
        Box::new(BooleanInfixTest {
            input: "false == false;".to_owned(),
            left_value: false,
            operator: "==".to_owned(),
            right_value: false,
        }),
    ];

    for test in tests.iter() {
        let program = parse_program_from(test.input());
        assert_eq!(program.statements.len(), 1);
        test.test_expression(&program);
    }
}

#[test]
fn test_if_expression() {
    let input = "if (x < y) { x }".to_owned();
    let program = parse_program_from(input);
    assert_eq!(program.statements.len(), 1);

    let if_expression = get_first_expression::<IfExpression>(&program);
    test_string_infix_expression(if_expression.condition.as_ref(), "x", "<", "y");
    let consequence = if_expression
        .consequence
        .statements
        .first()
        .and_then(|statement| statement.as_any().downcast_ref::<ExpressionStatement>())
        .unwrap();
    test_identifier(consequence.expression.as_ref(), "x".to_owned());
    assert!(if_expression.alternative.is_none());
}

#[test]
fn test_if_else_expression() {
    let input = "if (x < y) { x } else { y }".to_owned();
    let program = parse_program_from(input);
    assert_eq!(program.statements.len(), 1);

    let if_expression = get_first_expression::<IfExpression>(&program);
    test_string_infix_expression(if_expression.condition.as_ref(), "x", "<", "y");

    let consequence = if_expression
        .consequence
        .statements
        .first()
        .and_then(|statement| statement.as_any().downcast_ref::<ExpressionStatement>())
        .unwrap();
    test_identifier(consequence.expression.as_ref(), "x".to_owned());
    let alternative = if_expression
        .alternative
        .as_ref()
        .and_then(|alt| alt.statements.first())
        .and_then(|statement| statement.as_any().downcast_ref::<ExpressionStatement>())
        .unwrap();
    test_identifier(alternative.expression.as_ref(), "y".to_owned());
}

#[test]
fn test_function_literal_expression() {
    let input = "fn(x, y) { x + y; }".to_owned();
    let program = parse_program_from(input);
    assert_eq!(program.statements.len(), 1);

    let function_literal = get_first_expression::<FunctionLiteral>(&program);
    assert_eq!(function_literal.parameters.len(), 2);
    test_identifier(&function_literal.parameters[0], "x".to_owned());
    test_identifier(&function_literal.parameters[1], "y".to_owned());

    assert_eq!(function_literal.body.statements.len(), 1);
    let expression = function_literal.body.statements[0]
        .as_any()
        .downcast_ref::<ExpressionStatement>()
        .and_then(|expression_statement| {
            expression_statement
                .expression
                .as_any()
                .downcast_ref::<InfixExpression>()
        })
        .unwrap();
    test_string_infix_expression(expression, "x", "+", "y");
}

#[rstest]
#[case("fn() {}".to_owned(), Vec::new())]
#[case("fn(x) {}".to_owned(), vec!["x".to_owned()])]
#[case("fn(x, y, z) {}".to_owned(), vec!["x".to_owned(), "y".to_owned(), "z".to_owned()])]
fn test_function_parameter_parsing(#[case] input: String, #[case] expected_params: Vec<String>) {
    let program = parse_program_from(input);

    let function_literal = get_first_expression::<FunctionLiteral>(&program);
    assert_eq!(function_literal.parameters.len(), expected_params.len());
    for (index, param) in expected_params.into_iter().enumerate() {
        test_identifier(&function_literal.parameters[index], param);
    }
}

#[test]
fn test_call_expression_parsing() {
    let input = "add(1, 2 * 3, 4 + 5);".to_owned();

    let program = parse_program_from(input);

    let call_expression = get_first_expression::<CallExpression>(&program);

    test_identifier(call_expression.function.as_ref(), "add".to_owned());
    assert_eq!(call_expression.arguments.len(), 3);
    test_integer_literal(call_expression.arguments[0].as_ref(), 1);
    test_integer_infix_expression(call_expression.arguments[1].as_ref(), 2, "*", 3);
    test_integer_infix_expression(call_expression.arguments[2].as_ref(), 4, "+", 5);
}

#[test]
fn test_string_literal_expression() {
    let input = "\"hello world\"".to_owned();
    let program = parse_program_from(input);
    let literal = get_first_expression::<StringLiteral>(&program);
    assert_eq!(literal.string(), "hello world");
    assert_eq!(literal.value, "hello world");
}

#[test]
fn test_parsing_array_literal() {
    let input = "[1, 2 * 2, 3 + 3]".to_owned();
    let program = parse_program_from(input);
    let array = get_first_expression::<ArrayLiteral>(&program);
    assert_eq!(array.elements.len(), 3);

    test_integer_literal(array.elements[0].as_ref(), 1);
    test_integer_infix_expression(array.elements[1].as_ref(), 2, "*", 2);
    test_integer_infix_expression(array.elements[2].as_ref(), 3, "+", 3);
}

#[test]
fn test_parsing_index_expression() {
    let input = "myArray[1 + 1]".to_owned();
    let program = parse_program_from(input);
    let index_expression = get_first_expression::<IndexExpression>(&program);
    test_identifier(index_expression.left.as_ref(), "myArray".to_owned());
    test_integer_infix_expression(index_expression.index.as_ref(), 1, "+", 1);
}

#[test]
fn test_parsing_hash_literals_string_keys() {
    let input = r#"{"one": 1, "two": 2, "three": 3}"#.to_owned();
    let program = parse_program_from(input);
    let hash_literal = get_first_expression::<HashLiteral>(&program);
    assert_eq!(hash_literal.pairs.len(), 3);
    let expected: HashMap<&str, i64> = HashMap::from([("one", 1), ("two", 2), ("three", 3)]);
    for (key, value) in hash_literal.pairs.iter() {
        test_integer_literal(
            value.as_ref(),
            *expected.get(&key.string() as &str).unwrap(),
        );
    }
}

#[test]
fn test_parsing_empty_hash_literals() {
    let input = "{}".to_owned();
    let program = parse_program_from(input);
    let hash_literal = get_first_expression::<HashLiteral>(&program);
    assert_eq!(hash_literal.pairs.len(), 0);
}

#[test]
fn test_parsing_hash_literals_with_expressions() {
    let input = r#"{"one": 0 + 1, "two": 10 - 8, "three": 15 / 5}"#.to_owned();
    let program = parse_program_from(input);
    let hash_literal = get_first_expression::<HashLiteral>(&program);
    assert_eq!(hash_literal.pairs.len(), 3);
    type TestMapType<'a> = HashMap<&'a str, Box<dyn Fn(&dyn Expression)>>;
    let tests: TestMapType = HashMap::from([
        (
            "one",
            Box::new(|e: &dyn Expression| test_integer_infix_expression(e, 0, "+", 1))
                as Box<dyn Fn(&dyn Expression)>,
        ),
        (
            "two",
            Box::new(|e| test_integer_infix_expression(e, 10, "-", 8)),
        ),
        (
            "three",
            Box::new(|e| test_integer_infix_expression(e, 15, "/", 5)),
        ),
    ]);
    for (key, value) in hash_literal.pairs.iter() {
        let test_func = tests.get(&key.string() as &str).unwrap();
        test_func(value.as_ref());
    }
}
