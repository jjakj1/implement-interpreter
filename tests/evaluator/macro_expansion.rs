use std::{cell::RefCell, rc::Rc};

use implement_parser::{
    ast::traits::Node,
    evaluator::{
        environment::Environment,
        macro_expansion::{define_macros, expand_macro},
        object::Macro,
    },
};
use rstest::rstest;

use super::eval::parse_program_from;

#[test]
fn test_define_macro() {
    let input = "
        let number = 1;
        let function = fn(x, y) {x + y};
        let mymacro = macro(x, y) {x + y};"
        .to_owned();

    let mut program = parse_program_from(input);
    let env = Rc::new(RefCell::new(Environment::new()));
    define_macros(&mut program, Rc::clone(&env));

    assert_eq!(program.statements.len(), 2);
    assert!(env.borrow().get("number").is_none());
    assert!(env.borrow().get("function").is_none());
    assert!(env.borrow().get("mymacro").is_some());

    let object = env.borrow().get("mymacro").unwrap();
    let macro_object = object.downcast_ref::<Macro>().unwrap();
    assert_eq!(macro_object.parameters.len(), 2);
    assert_eq!(macro_object.parameters[0].string(), "x");
    assert_eq!(macro_object.parameters[1].string(), "y");
    assert_eq!(macro_object.body.string(), "(x + y)");
}

#[rstest]
#[case(r#"let infixExpression = macro() { quote(1 + 2); }; infixExpression();"#.to_owned(), "(1 + 2)".to_owned())]
#[case(r#"let reverse = macro(a, b) { quote(unquote(b) - unquote(a)); }; reverse(2 + 2, 10 - 5);"#.to_owned(), "(10 - 5) - (2 + 2)".to_owned())]
#[case(r#"let unless = macro(condition, consequence, alternative) {
 quote(
if (!(unquote(condition))) {
    unquote(consequence);
} else {
    unquote(alternative);
});
}; unless(10 > 5, puts("not greater"), puts("greater")); "#.to_owned(), r#"if (!(10 > 5)) { puts("not greater") } else { puts("greater") }"#.to_owned())]
fn test_expand_macro(#[case] input: String, #[case] expected: String) {
    let expected = parse_program_from(expected);
    let mut program = parse_program_from(input);
    let env = Rc::new(RefCell::new(Environment::new()));
    define_macros(&mut program, Rc::clone(&env));
    let expanded = expand_macro(&mut program, Rc::clone(&env));
    assert_eq!(expanded.string(), expected.string());
}
