use crate::{
    ast::{
        expressions::{self, CallExpression, IntegerLiteral},
        modify::modify,
        traits::Node,
    },
    evaluator::{
        environment::Environment,
        object::{self, Integer, Object, Quote},
    },
    token::{Token, TokenType},
};
use std::{cell::RefCell, rc::Rc};

pub fn quote(node: &mut Box<dyn Node>, environment: Rc<RefCell<Environment>>) -> Box<dyn Object> {
    let new_node = eval_unquote_calls(node.as_mut_node(), environment);
    Box::new(Quote { node: new_node })
}

// 没有办法用一个 &dyn Node 的内容去替换另一个，因为都不知道 dyn Node 具体类型的大小，也就不知道要复制多少过去
// https://stackoverflow.com/questions/25246443/how-can-i-downcast-from-boxany-to-a-trait-object-type
fn eval_unquote_calls(node: &mut dyn Node, environment: Rc<RefCell<Environment>>) -> Box<dyn Node> {
    modify(node, &|node| {
        if !is_unquote_call(node.as_ref()) {
            return node;
        }
        if let Some(expression) = node.downcast_ref::<CallExpression>() {
            if expression.arguments.len() == 1 {
                let new_node = convert_object_to_ast_node(
                    expression.arguments[0].eval_to_object(Rc::clone(&environment)),
                );
                return new_node;
            }
        }
        node
    })
}

fn is_unquote_call(node: &dyn Node) -> bool {
    node.downcast_ref::<CallExpression>()
        .map(|expression| expression.function.string() == "unquote")
        .unwrap_or_default()
}

fn convert_object_to_ast_node(object: Box<dyn Object>) -> Box<dyn Node> {
    if let Some(integer) = object.downcast_ref::<Integer>() {
        let token = Token {
            token_type: TokenType::Int,
            literal: format!("{}", integer.value),
        };
        Box::new(IntegerLiteral {
            token,
            value: integer.value,
        })
    } else if let Some(boolean) = object.downcast_ref::<object::Boolean>() {
        let token = if matches!(boolean, object::Boolean::True) {
            Token {
                token_type: TokenType::True,
                literal: "true".to_owned(),
            }
        } else {
            Token {
                token_type: TokenType::False,
                literal: "false".to_owned(),
            }
        };
        Box::new(expressions::Boolean {
            token,
            value: boolean.value(),
        })
    } else if let Some(quote) = object.downcast_ref::<object::Quote>() {
        dyn_clone::clone_box(quote.node.as_ref())
    } else {
        panic!("other object types are not supported");
    }
}
