use super::environment::Environment;
use super::object::{
    self, Boolean, HashPair, Hashable, Integer, Object, ObjectType, StringObject, BUILTINS,
};
use crate::ast::expressions::{HashLiteral, Identifier};
use crate::ast::program::Program;
use crate::ast::statements::BlockStatement;
use crate::ast::traits::{AsNode, Expression, Node};
use std::collections::HashMap;
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
                let return_object = result?.downcast::<object::ReturnValue>().ok()?;
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

// TODO: 把 Option 改成 Result
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
        let left_integer = left.downcast_ref::<Integer>()?;
        let right_integer = right.downcast_ref::<Integer>()?;
        eval_integer_infix_expression(left_integer, operator, right_integer)
    } else if matches!(left.object_type(), ObjectType::Boolean)
        && matches!(right.object_type(), ObjectType::Boolean)
    {
        let left_boolean = left.as_any().downcast_ref::<Boolean>()?;
        let right_boolean = right.as_any().downcast_ref::<Boolean>()?;
        eval_boolean_infix_expression(left_boolean, operator, right_boolean)
    } else if matches!(left.object_type(), ObjectType::String)
        && matches!(right.object_type(), ObjectType::String)
    {
        let left_string = left.as_any().downcast_ref::<StringObject>()?;
        let right_string = right.as_any().downcast_ref::<StringObject>()?;
        eval_string_infix_expression(left_string, operator, right_string)
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
    exps: &[Box<dyn Expression>],
    env: Rc<RefCell<Environment>>,
) -> Option<Vec<Box<dyn Object>>> {
    let mut results = Vec::new();
    for exp in exps {
        let object = eval(exp.as_node(), env.clone());
        if is_error(object.as_deref()) {
            return Some(vec![object?]);
        }
        if let Some(res) = object {
            results.push(res)
        }
    }
    Some(results)
}

pub fn eval_identifier(
    identifier: &Identifier,
    env: Rc<RefCell<Environment>>,
) -> Option<Box<dyn Object>> {
    env.borrow()
        .get(&identifier.value)
        .or_else(|| {
            BUILTINS
                .get(&*identifier.value) // https://stackoverflow.com/questions/65549983/trait-borrowstring-is-not-implemented-for-str
                .map(|buildin| dyn_clone::clone_box(buildin) as Box<dyn Object>)
        })
        .or(Some(Box::new(object::Error {
            message: format!("identifier not found: {}", identifier.value),
        })))
}

pub fn eval_index_expression(
    left: Option<Box<dyn Object>>,
    index: Option<Box<dyn Object>>,
) -> Option<Box<dyn Object>> {
    let left = left?;
    let index = index?;
    let left_type = left.object_type();
    if matches!(left.object_type(), ObjectType::Array)
        && matches!(index.object_type(), ObjectType::Integer)
    {
        let array = left.downcast_ref::<object::Array>()?;
        let index = index.downcast_ref::<object::Integer>()?;
        if array.elements.len() <= index.value as usize || index.value < 0 {
            return Some(Box::new(object::Null));
        }

        return Some(dyn_clone::clone_box(
            array.elements[index.value as usize].as_ref(),
        ));
    } else if matches!(left.object_type(), ObjectType::Hash) {
        let hash = left.downcast_ref::<object::Hash>()?;
        return eval_hash_index_expression(hash, index.as_ref());
    }

    Some(Box::new(object::Error {
        message: format!("index operator not supported: {:?}", left_type),
    }))
}

pub fn eval_hash_literal(
    node: &HashLiteral,
    env: Rc<RefCell<Environment>>,
) -> Option<Box<dyn Object>> {
    let mut pairs = HashMap::new();
    for (key, value) in node.pairs.iter() {
        let evaluated_key = eval(key.as_node(), Rc::clone(&env));
        if is_error(evaluated_key.as_deref()) {
            return evaluated_key;
        }
        let evaluated_value = eval(value.as_node(), Rc::clone(&env));
        if is_error(evaluated_value.as_deref()) {
            return evaluated_value;
        }
        let evaluated_key = evaluated_key?;
        let evaluated_value = evaluated_value?;
        match evaluated_key.object_type() {
            object::ObjectType::String => {
                let str = evaluated_key.downcast::<object::StringObject>().ok()?;
                pairs.insert(
                    str.hash_key(),
                    HashPair {
                        key: str,
                        value: evaluated_value,
                    },
                );
            }
            object::ObjectType::Integer => {
                let integer = evaluated_key.downcast::<object::Integer>().ok()?;
                pairs.insert(
                    integer.hash_key(),
                    HashPair {
                        key: integer,
                        value: evaluated_value,
                    },
                );
            }
            object::ObjectType::Boolean => {
                let boolean = evaluated_key.downcast::<object::Boolean>().ok()?;
                pairs.insert(
                    boolean.hash_key(),
                    HashPair {
                        key: boolean,
                        value: evaluated_value,
                    },
                );
            }
            _ => {
                return Some(Box::new(object::Error {
                    message: format!("unusable as hash key: {:?}", evaluated_key.object_type()),
                }));
            }
        };
    }
    Some(Box::new(object::Hash { pairs }))
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

pub fn is_error(object: Option<&dyn Object>) -> bool {
    if let Some(object) = object {
        matches!(object.object_type(), ObjectType::Error)
    } else {
        false
    }
}

pub fn apply_function(func: Box<dyn Object>, args: &[Box<dyn Object>]) -> Option<Box<dyn Object>> {
    let func_type = func.object_type();
    match func.object_type() {
        ObjectType::Function => {
            let f = func.downcast_ref::<object::Function>()?;
            let env = extend_function_env(f, args);
            let object = eval(f.body.as_node(), Rc::new(RefCell::new(env)))?;
            Some(unwrap_return_value(object))
        }
        ObjectType::Builtin => {
            let f = func.downcast_ref::<object::Builtin>()?;
            let args = args.iter().map(Box::as_ref).collect::<Vec<_>>();
            Some((f.func)(&args))
        }
        _ => Some(Box::new(object::Error {
            message: format!("not a function: {:?}", func_type),
        })),
    }
}

fn extend_function_env(func: &object::Function, args: &[Box<dyn Object>]) -> Environment {
    let mut enclosed_env = Environment::new_enclosed(Rc::downgrade(&func.env));

    for (index, param) in func.parameters.iter().enumerate() {
        enclosed_env.set(
            param.value.clone(),
            dyn_clone::clone_box(args[index].as_ref()),
        );
    }

    enclosed_env
}

fn unwrap_return_value(object: Box<dyn Object>) -> Box<dyn Object> {
    if matches!(object.object_type(), ObjectType::ReturnValue) {
        return object
            .downcast::<object::ReturnValue>()
            .map_err(|_| "Shouldn't happen.")
            .unwrap();
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

    let integer = right.downcast_ref::<Integer>()?;
    Some(Box::new(Integer {
        value: -integer.value,
    }))
}

fn eval_integer_infix_expression(
    left: &Integer,
    operator: &str,
    right: &Integer,
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
    left: &Boolean,
    operator: &str,
    right: &Boolean,
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

fn eval_string_infix_expression(
    left: &StringObject,
    operator: &str,
    right: &StringObject,
) -> Option<Box<dyn Object>> {
    match operator {
        "+" => Some(Box::new(StringObject {
            value: left.value.clone() + &right.value,
        })),
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

fn eval_hash_index_expression(hash: &object::Hash, index: &dyn Object) -> Option<Box<dyn Object>> {
    let hash_key = match index.object_type() {
        ObjectType::String => {
            let str = index.downcast_ref::<object::StringObject>()?;
            str.hash_key()
        }
        ObjectType::Integer => {
            let integer = index.downcast_ref::<object::Integer>()?;
            integer.hash_key()
        }
        ObjectType::Boolean => {
            let boolean = index.downcast_ref::<object::Boolean>()?;
            boolean.hash_key()
        }
        _ => {
            return Some(Box::new(object::Error {
                message: format!("unusable as hash key: {:?}", index.object_type()),
            }));
        }
    };
    return hash
        .pairs
        .get(&hash_key)
        .map_or(Some(Box::new(object::Null)), |value| {
            Some(dyn_clone::clone_box(value.value.as_ref()))
        });
}
