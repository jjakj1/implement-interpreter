use std::cell::RefCell;
use std::rc::Rc;

use implement_parser::ast::program::Program;
use implement_parser::ast::traits::Node;
use implement_parser::environment::Environment;
use implement_parser::evaluator::eval;
use implement_parser::lexer::Lexer;
use implement_parser::object::{Boolean, Error, Function, Integer, Null, Object};
use implement_parser::parser::Parser;
use rstest::rstest;

// TODO：我本来想要把 parser 里面的 helpers 拿出来放在 tests 目录下，然后在这里引用。但试了下好像引用不到。目前我感觉好像 tests 中的各个文件都是一个单独的 crate 不能交叉引用。但不是很确定
fn parse_program_from(input: String) -> Program {
    let lexer = Lexer::new(input);
    let mut parser = Parser::new(lexer);
    let program = parser.parse_program();
    for err in parser.error_messages {
        eprintln!("{}", err);
    }
    program
}

fn test_eval(input: String) -> Box<dyn Object> {
    let program = parse_program_from(input);
    let env = Environment::new();
    eval(&program, Rc::new(RefCell::new(env))).unwrap()
}

#[rstest]
#[case("5".to_owned(), 5)]
#[case("10".to_owned(), 10)]
#[case::prefix("-5".to_owned(), -5)]
#[case::prefix("-10".to_owned(), -10)]
#[case::infix("5 + 5 + 5 + 5 - 10".to_owned(), 10)]
#[case::infix("2 * 2 * 2 * 2 * 2".to_owned(), 32)]
#[case::infix("-50 + 100 + -50".to_owned(), 0)]
#[case::infix("5 * 2 + 10".to_owned(), 20)]
#[case::infix("5 + 2 * 10".to_owned(), 25)]
#[case::infix("20 + 2 * -10".to_owned(), 0)]
#[case::infix("50 / 2 * 2 + 10".to_owned(), 60)]
#[case::infix("2 * (5 + 10)".to_owned(), 30)]
#[case::infix("3 * 3 * 3 + 10".to_owned(), 37)]
#[case::infix("3 * (3 * 3) + 10".to_owned(), 37)]
#[case::infix("(5 + 10 * 2 + 15 / 3) * 2 + -10".to_owned(), 50)]
fn test_eval_integer_expression(#[case] input: String, #[case] expected: i64) {
    let object = test_eval(input);
    let integer = object.as_any().downcast::<Integer>().unwrap();
    assert_eq!(integer.value, expected);
}

#[rstest]
#[case("true".to_owned(), true)]
#[case("false".to_owned(), false)]
#[case::infix("1 < 2".to_owned(), true)]
#[case::infix("1 > 2".to_owned(), false)]
#[case::infix("1 < 1".to_owned(), false)]
#[case::infix("1 > 1".to_owned(), false)]
#[case::infix("1 == 1".to_owned(), true)]
#[case::infix("1 != 1".to_owned(), false)]
#[case::infix("1 == 2".to_owned(), false)]
#[case::infix("1 != 2".to_owned(), true)]
#[case::infix("true == true".to_owned(), true)]
#[case::infix("false == false".to_owned(), true)]
#[case::infix("true == false".to_owned(), false)]
#[case::infix("true != false".to_owned(), true)]
#[case::infix("false != true".to_owned(), true)]
#[case::infix("(1 < 2) == true".to_owned(), true)]
#[case::infix("(1 < 2) == false".to_owned(), false)]
#[case::infix("(1 > 2) == true".to_owned(), false)]
#[case::infix("(1 > 2) == false".to_owned(), true)]
fn test_eval_boolean_expression(#[case] input: String, #[case] expected: bool) {
    let object = test_eval(input);
    let boolean = object.as_any().downcast::<Boolean>().unwrap();
    assert_eq!(boolean.value(), expected);
}

#[rstest]
#[case("if (true) { 10 }".to_owned(), Some(10))]
#[case("if (false) { 10 }".to_owned(), None)]
#[case("if (1) { 10 }".to_owned(), Some(10))]
#[case("if (1 < 2) { 10 }".to_owned(), Some(10))]
#[case("if (1 > 2) { 10 }".to_owned(), None)]
#[case("if (1 > 2) { 10 } else { 20 }".to_owned(), Some(20))]
#[case("if (1 < 2) { 10 } else { 20 }".to_owned(), Some(10))]
fn test_if_else_expression(#[case] input: String, #[case] expected: Option<i64>) {
    let object = test_eval(input);
    if let Some(expected) = expected {
        let integer = object.as_any().downcast::<Integer>().unwrap();
        assert_eq!(integer.value, expected);
    } else {
        assert!(object.as_any().downcast_ref::<Null>().is_some());
    }
}

#[rstest]
#[case("!true".to_owned(), false)]
#[case("!false".to_owned(), true)]
#[case("!5".to_owned(), false)]
#[case("!!true".to_owned(), true)]
#[case("!!false".to_owned(), false)]
#[case("!!5".to_owned(), true)]
fn test_bang_operator(#[case] input: String, #[case] expected: bool) {
    let object = test_eval(input);
    let boolean = object.as_any().downcast::<Boolean>().unwrap();
    assert_eq!(boolean.value(), expected);
}

#[rstest]
#[case("return 10;".to_owned(), 10)]
#[case("return 10; 9;".to_owned(), 10)]
#[case("return 2 * 5; 9;".to_owned(), 10)]
#[case("9; return 10; 9;".to_owned(), 10)]
#[case::nested("if (10 > 1) { if (10 > 1) { return 10; } return 1; }".to_owned(), 10)]
fn test_return_statement(#[case] input: String, #[case] expected: i64) {
    let object = test_eval(input);
    let integer = object.as_any().downcast::<Integer>().unwrap();
    assert_eq!(integer.value, expected);
}

#[rstest]
#[case("let a = 5; a;".to_owned(), 5)]
#[case("let a = 5 * 5; a;".to_owned(), 25)]
#[case("let a = 5; let b = a; b;".to_owned(), 5)]
#[case("let a = 5; let b = a; let c = a + b + 5; c;".to_owned(), 15)]
fn test_let_statements(#[case] input: String, #[case] expected: i64) {
    let object = test_eval(input);
    let integer = object.as_any().downcast::<Integer>().unwrap();
    assert_eq!(integer.value, expected);
}

#[rstest]
#[case("5 + true;".to_owned(), "type mismatch: Integer + Boolean".to_owned())]
#[case("5 + true; 5;".to_owned(), "type mismatch: Integer + Boolean".to_owned())]
#[case("-true".to_owned(), "unknown operator: -Boolean".to_owned())]
#[case("true + false;".to_owned(), "unknown operator: Boolean + Boolean".to_owned())]
#[case("5; true + false; 5".to_owned(), "unknown operator: Boolean + Boolean".to_owned())]
#[case("if (10 > 1) { true + false; }".to_owned(), "unknown operator: Boolean + Boolean".to_owned())]
#[case("if (10 > 1) { if (10 > 1) { return true + false; } return 1; }".to_owned(), "unknown operator: Boolean + Boolean".to_owned())]
#[case("foobar".to_owned(), "identifier not found: foobar".to_owned())]
fn test_error_handling(#[case] input: String, #[case] expected_message: String) {
    let object = test_eval(input);
    let error = object.as_any().downcast::<Error>().unwrap();
    assert_eq!(error.message, expected_message);
}

#[test]
fn test_function_object() {
    let input = "fn(x) { x + 2; };".to_owned();
    let evaluated = test_eval(input);
    let function = evaluated.as_any().downcast::<Function>().unwrap();
    assert_eq!(function.parameters.len(), 1);
    assert_eq!(function.parameters[0].string(), "x");
    assert_eq!(function.body.string(), "(x + 2)");
}

#[rstest]
#[case("let identity = fn(x) { x; }; identity(5);".to_owned(), 5)]
#[case("let identity = fn(x) { return x; }; identity(5);".to_owned(), 5)]
#[case("let double = fn(x) { x * 2; }; double(5);".to_owned(), 10)]
#[case("let add = fn(x, y) { x + y; }; add(5, 5);".to_owned(), 10)]
#[case("let add = fn(x, y) { x + y; }; add(5 + 5, add(5, 5));".to_owned(), 20)]
#[case("fn(x) { x; }(5)".to_owned(), 5)]
fn test_function_application(#[case] input: String, #[case] expected: i64) {
    let evaluated = test_eval(input);
    let integer = evaluated.as_any().downcast::<Integer>().unwrap();
    assert_eq!(integer.value, expected);
}

#[test]
fn test_closures() {
    let input = "
    let newAdder = fn(x) {
        fn(y) { x + y };
    };
    let addTwo = newAdder(2);
    addTwo(2);"
        .to_owned();

    let object = test_eval(input);
    let integer = object.as_any().downcast::<Integer>().unwrap();
    assert_eq!(integer.value, 4);
}
