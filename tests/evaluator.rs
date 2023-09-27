use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use implement_parser::ast::program::Program;
use implement_parser::ast::traits::Node;
use implement_parser::evaluator::environment::Environment;
use implement_parser::evaluator::eval::eval;
use implement_parser::evaluator::object::{
    self, Array, Boolean, Error, Function, HashKey, Hashable, Integer, Null, Object, ObjectType,
    StringObject,
};
use implement_parser::lexer::Lexer;
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
    eval(&program, Rc::new(RefCell::new(env)))
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
    let integer = object.downcast_ref::<Integer>().unwrap();
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
    let boolean = object.downcast_ref::<Boolean>().unwrap();
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
        let integer = object.downcast_ref::<Integer>().unwrap();
        assert_eq!(integer.value, expected);
    } else {
        assert!(object.downcast_ref::<Null>().is_some());
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
    let boolean = object.downcast_ref::<Boolean>().unwrap();
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
    let integer = object.downcast_ref::<Integer>().unwrap();
    assert_eq!(integer.value, expected);
}

#[rstest]
#[case("let a = 5; a;".to_owned(), 5)]
#[case("let a = 5 * 5; a;".to_owned(), 25)]
#[case("let a = 5; let b = a; b;".to_owned(), 5)]
#[case("let a = 5; let b = a; let c = a + b + 5; c;".to_owned(), 15)]
fn test_let_statements(#[case] input: String, #[case] expected: i64) {
    let object = test_eval(input);
    let integer = object.downcast_ref::<Integer>().unwrap();
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
#[case("\"Hello\" - \"World!\"".to_owned(), "unknown operator: String - String".to_owned())]
fn test_error_handling(#[case] input: String, #[case] expected_message: String) {
    let object = test_eval(input);
    let error = object.downcast_ref::<Error>().unwrap();
    assert_eq!(error.message, expected_message);
}

#[test]
fn test_function_object() {
    let input = "fn(x) { x + 2; };".to_owned();
    let evaluated = test_eval(input);
    let function = evaluated.downcast_ref::<Function>().unwrap();
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
    let integer = evaluated.downcast_ref::<Integer>().unwrap();
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
    let integer = object.downcast_ref::<Integer>().unwrap();
    assert_eq!(integer.value, 4);
}

#[test]
fn test_string_literal() {
    let input = "\"Hello World!".to_owned();
    let evaluated = test_eval(input);
    let string = evaluated.downcast_ref::<StringObject>().unwrap();
    assert_eq!(string.value, "Hello World!");
}

#[test]
fn test_string_concatenation() {
    let input = r#""Hello" + " " + "World!""#.to_owned();
    let evaluated = test_eval(input);
    let string = evaluated.downcast_ref::<StringObject>().unwrap();
    assert_eq!(string.value, "Hello World!");
}

#[rstest]
#[case(r#"len("")"#.to_owned(), "0".to_owned())]
#[case(r#"len("four")"#.to_owned(), "4".to_owned())]
#[case(r#"len("hello world")"#.to_owned(), "11".to_owned())]
#[case(r#"len(1)"#.to_owned(), "argument to `len` not supported, got Integer".to_owned())]
#[case(r#"len("one", "one")"#.to_owned(), "wrong number of arguments: got=2, want=1".to_owned())]
fn test_builtin_functions(#[case] input: String, #[case] expected: String) {
    let evaluated = test_eval(input);
    match evaluated.object_type() {
        ObjectType::Integer => {
            let integer = evaluated.downcast_ref::<Integer>().unwrap();
            assert_eq!(integer.value.to_string(), expected);
        }
        ObjectType::Error => {
            let error = evaluated.downcast_ref::<Error>().unwrap();
            assert_eq!(error.message, expected);
        }
        _ => {
            panic!("object is of type: {:?}", evaluated.object_type());
        }
    }
}

#[test]
fn test_array_literals() {
    let input = "[1, 2 * 2, 3 + 3]".to_owned();
    let evaluated = test_eval(input);
    let array = evaluated.downcast_ref::<Array>().unwrap();
    assert_eq!(array.elements.len(), 3);

    let first = array.elements[0].downcast_ref::<Integer>().unwrap();
    assert_eq!(first.value, 1);
    let second = array.elements[1].downcast_ref::<Integer>().unwrap();
    assert_eq!(second.value, 4);
    let third = array.elements[2].downcast_ref::<Integer>().unwrap();
    assert_eq!(third.value, 6);
}

#[rstest]
#[case("[1, 2, 3][0]".to_owned(), Some(1))]
#[case("[1, 2, 3][1]".to_owned(), Some(2))]
#[case("[1, 2, 3][2]".to_owned(), Some(3))]
#[case("let i = 0; [1][i];".to_owned(), Some(1))]
#[case("[1, 2, 3][1 + 1]".to_owned(), Some(3))]
#[case("let myArray = [1, 2, 3]; myArray[2];".to_owned(), Some(3))]
#[case("let myArray = [1, 2, 3]; myArray[0] + myArray[1] + myArray[2];".to_owned(), Some(6))]
#[case("let myArray = [1, 2, 3]; let i = myArray[0]; myArray[i];".to_owned(), Some(2))]
#[case("[1, 2, 3][3]".to_owned(), None)]
#[case("[1, 2, 3][-1]".to_owned(), None)]
fn test_array_index_expression(#[case] input: String, #[case] expected: Option<i64>) {
    let evaluated = test_eval(input);
    if let Some(expected) = expected {
        let integer = evaluated.downcast_ref::<Integer>().unwrap();
        assert_eq!(integer.value, expected);
    } else {
        assert!(evaluated.as_any().downcast_ref::<Null>().is_some())
    }
}

#[test]
fn test_hash_literals() {
    let input = r#"let two = "two";
        {
            "one": 10 - 9,
            two: 1 + 1,
            "thr" + "ee": 6 / 2,
            4: 4,
            true: 5,
            false: 6
        }
        "#
    .to_owned();
    let mut evaluated = test_eval(input);
    let hash = evaluated.downcast_mut::<object::Hash>().unwrap();
    let expected: HashMap<HashKey, i64> = HashMap::from([
        (
            StringObject {
                value: "one".to_owned(),
            }
            .hash_key(),
            1,
        ),
        (
            StringObject {
                value: "two".to_owned(),
            }
            .hash_key(),
            2,
        ),
        (
            StringObject {
                value: "three".to_owned(),
            }
            .hash_key(),
            3,
        ),
        (Integer { value: 4 }.hash_key(), 4),
        (Boolean::True.hash_key(), 5),
        (Boolean::False.hash_key(), 6),
    ]);
    assert_eq!(hash.pairs.len(), expected.len());
    for (expected_key, expected_value) in expected {
        let pair = hash.pairs.remove(&expected_key).unwrap();
        let integer = pair.value.downcast_ref::<Integer>().unwrap();
        assert_eq!(integer.value, expected_value);
    }
}

#[rstest]
#[case(r#"{"foo": 5}["foo"]"#.to_owned(), Some(5))]
#[case(r#"{"foo": 5}["bar"]"#.to_owned(), None)]
#[case(r#"let key = "foo"; {"foo": 5}[key]"#.to_owned(), Some(5))]
#[case(r#"{}["foo"]"#.to_owned(), None)]
#[case(r#"{5: 5}[5]"#.to_owned(), Some(5))]
#[case(r#"{true: 5}[true]"#.to_owned(), Some(5))]
#[case(r#"{false: 5}[false]"#.to_owned(), Some(5))]
fn test_hash_index_expression(#[case] input: String, #[case] expected: Option<i64>) {
    let evaluated = test_eval(input);
    if let Some(expected) = expected {
        let value = evaluated.downcast_ref::<object::Integer>().unwrap();
        assert_eq!(value.value, expected);
    } else {
        assert!(evaluated.downcast_ref::<object::Null>().is_some());
    }
}
