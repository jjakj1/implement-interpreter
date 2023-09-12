use crate::ast::program::Program;
use crate::ast::statements::BlockStatement;
use crate::ast::traits::{AsNode, Expression, Node};
use crate::environment::Environment;
use crate::object::{self, Boolean, Integer, Object, ObjectType};
use std::{cell::RefCell, rc::Rc};

// TODO: Rust 里面好像不允许对一个 dynamic dispatch 的类型做判断，但我不太确定：https://www.reddit.com/r/rust/comments/ajd0je/how_to_get_type_of_a_boximpl_trait/
// 所以我这里扩展了之前的 node trait
pub fn eval(node: &dyn Node, env: Rc<RefCell<Environment>>) -> Option<Box<dyn Object>> {
    node.eval_to_object(env)
}

pub fn eval_program(program: &Program, env: Rc<RefCell<Environment>>) -> Option<Box<dyn Object>> {
    let mut result = None;
    for statement in program.statements.iter() {
        result = eval(statement.as_node(), env.clone());
        if let Some(object) = &result {
            if matches!(object.object_type(), ObjectType::ReturnValue) {
                let return_object = result?
                    .as_any()
                    .downcast::<crate::object::ReturnValue>()
                    .ok()?;
                return Some(return_object.value);
            } else if matches!(object.object_type(), ObjectType::Error) {
                return result;
            }
        }
    }
    result
}

pub fn eval_block_statement(
    block_statement: &BlockStatement,
    env: Rc<RefCell<Environment>>,
) -> Option<Box<dyn Object>> {
    let mut result = None;
    for statement in block_statement.statements.iter() {
        result = eval(statement.as_node(), env.clone());
        if let Some(object) = &result {
            if matches!(object.object_type(), ObjectType::ReturnValue)
                || matches!(object.object_type(), ObjectType::Error)
            {
                return result;
            }
        }
    }
    result
}

pub fn eval_prefix_expression(
    operator: &str,
    right: Option<Box<dyn Object>>,
) -> Option<Box<dyn Object>> {
    match operator {
        "!" => eval_bang_operator_expression(right),
        "-" => eval_minus_prefix_operator_expression(right),
        _ => Some(Box::new(object::Error {
            message: format!("unknown operator: {}{:?}", operator, right?.object_type()),
        })),
    }
}

pub fn eval_infix_expression(
    left: Option<Box<dyn Object>>,
    operator: &str,
    right: Option<Box<dyn Object>>,
) -> Option<Box<dyn Object>> {
    let left = left?;
    let right = right?;
    if matches!(left.object_type(), ObjectType::Integer)
        && matches!(right.object_type(), ObjectType::Integer)
    {
        let left_integer = left.as_any().downcast::<Integer>().ok()?;
        let right_integer = right.as_any().downcast::<Integer>().ok()?;
        eval_integer_infix_expression(*left_integer, operator, *right_integer)
    } else if matches!(left.object_type(), ObjectType::Boolean)
        && matches!(right.object_type(), ObjectType::Boolean)
    {
        let left_boolean = left.as_any().downcast::<Boolean>().ok()?;
        let right_boolean = right.as_any().downcast::<Boolean>().ok()?;
        eval_boolean_infix_expression(*left_boolean, operator, *right_boolean)
    } else if left.object_type() != right.object_type() {
        Some(Box::new(object::Error {
            message: format!(
                "type mismatch: {:?} {} {:?}",
                left.object_type(),
                operator,
                right.object_type(),
            ),
        }))
    } else {
        Some(Box::new(object::Error {
            message: format!(
                "unknown operator: {:?} {} {:?}",
                left.object_type(),
                operator,
                right.object_type()
            ),
        }))
    }
}

pub fn eval_expressions(
    exps: &Vec<Box<dyn Expression>>,
    env: Rc<RefCell<Environment>>,
) -> Option<Vec<Box<dyn Object>>> {
    let mut results = Vec::new();
    for exp in exps {
        let object = eval(exp.as_node(), env.clone());
        if is_error(&object) {
            return Some(vec![object?]);
        }
        if let Some(res) = object {
            results.push(res)
        }
    }
    Some(results)
}

pub fn is_truthy(object: Option<Box<dyn Object>>) -> Option<bool> {
    let object = object?;
    match object.object_type() {
        ObjectType::Boolean => match object.as_any().downcast_ref::<Boolean>()? {
            Boolean::False => Some(false),
            Boolean::True => Some(true),
        },
        ObjectType::Null => Some(true),
        _ => Some(true),
    }
}

pub fn is_error(object: &Option<Box<dyn Object>>) -> bool {
    if let Some(object) = object {
        matches!(object.object_type(), ObjectType::Error)
    } else {
        false
    }
}

// TODO: 感觉很多地方的 Option 可以去掉
pub fn apply_function(
    func: Box<dyn Object>,
    args: Vec<Box<dyn Object>>,
) -> Option<Box<dyn Object>> {
    let func_type = func.object_type();
    if let Ok(f) = func.as_any().downcast::<object::Function>() {
        let env = extend_function_env(f.as_ref(), args);
        let object = eval(f.body.as_node(), Rc::new(RefCell::new(env)))?;
        return Some(unwrap_return_value(object));
    }

    Some(Box::new(object::Error {
        message: format!("not a function: {:?}", func_type),
    }))
}

fn extend_function_env(func: &object::Function, args: Vec<Box<dyn Object>>) -> Environment {
    let mut enclosed_env = Environment::new_enclosed(Rc::downgrade(&func.env));

    for (index, param) in func.parameters.iter().enumerate() {
        // TODO: 这个地方好像只能 clone，如果想要 swap_remove 需要 args 是可变的
        enclosed_env.set(
            param.value.clone(),
            dyn_clone::clone_box(args[index].as_ref()),
        );
    }

    enclosed_env
}

fn unwrap_return_value(object: Box<dyn Object>) -> Box<dyn Object> {
    if matches!(object.object_type(), ObjectType::ReturnValue) {
        return object.as_any().downcast::<object::ReturnValue>().unwrap();
    }

    object
}

fn eval_bang_operator_expression(right: Option<Box<dyn Object>>) -> Option<Box<dyn Object>> {
    if is_truthy(right)? {
        Some(Box::new(Boolean::False))
    } else {
        Some(Box::new(Boolean::True))
    }
}

fn eval_minus_prefix_operator_expression(
    right: Option<Box<dyn Object>>,
) -> Option<Box<dyn Object>> {
    let right = right?;
    if !matches!(right.object_type(), ObjectType::Integer) {
        return Some(Box::new(object::Error {
            message: format!("unknown operator: -{:?}", right.object_type()),
        }));
    }

    let integer = right.as_any().downcast::<Integer>().ok()?;
    Some(Box::new(Integer {
        value: -integer.value,
    }))
}

// TODO: 有一个 lint 告诉我 local variable doesn't need to be boxed here，所以我改成了 Integer，但下面的方法又没有提醒
fn eval_integer_infix_expression(
    left: Integer,
    operator: &str,
    right: Integer,
) -> Option<Box<dyn Object>> {
    match operator {
        "+" => Some(Box::new(Integer {
            value: left.value + right.value,
        })),
        "-" => Some(Box::new(Integer {
            value: left.value - right.value,
        })),
        "*" => Some(Box::new(Integer {
            value: left.value * right.value,
        })),
        "/" => Some(Box::new(Integer {
            value: left.value / right.value,
        })),
        "<" => Some(Box::new(Boolean::from_native_bool(
            left.value < right.value,
        ))),
        ">" => Some(Box::new(Boolean::from_native_bool(
            left.value > right.value,
        ))),
        "==" => Some(Box::new(Boolean::from_native_bool(
            left.value == right.value,
        ))),
        "!=" => Some(Box::new(Boolean::from_native_bool(
            left.value != right.value,
        ))),
        _ => Some(Box::new(object::Error {
            message: format!(
                "unknown operator: {:?} {} {:?}",
                left.object_type(),
                operator,
                right.object_type()
            ),
        })),
    }
}

fn eval_boolean_infix_expression(
    left: Boolean,
    operator: &str,
    right: Boolean,
) -> Option<Box<dyn Object>> {
    match operator {
        "==" => Some(Box::new(Boolean::from_native_bool(left == right))),
        "!=" => Some(Box::new(Boolean::from_native_bool(left != right))),
        _ => Some(Box::new(object::Error {
            message: format!(
                "unknown operator: {:?} {} {:?}",
                left.object_type(),
                operator,
                right.object_type()
            ),
        })),
    }
}
