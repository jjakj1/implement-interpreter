use std::{cell::RefCell, rc::Rc};

use crate::ast::{
    expressions::{CallExpression, Identifier, MacroLiteral},
    modify::modify,
    program::Program,
    statements::LetStatement,
    traits::{AsNode, Node, Statement},
};

use super::{
    environment::Environment,
    eval::eval,
    object::{Macro, Quote},
};

pub fn define_macros(program: &mut Program, env: Rc<RefCell<Environment>>) {
    let mut macro_indices = vec![];
    for (i, statement) in program.statements.iter().enumerate() {
        if is_macro_definiation(statement.as_ref()) {
            macro_indices.push(i);
            add_macro(statement.as_ref(), Rc::clone(&env));
        }
    }

    for index in macro_indices.iter().rev() {
        program.statements.remove(*index);
    }
}

pub fn expand_macro(program: &mut Program, env: Rc<RefCell<Environment>>) -> Box<dyn Node> {
    modify(program, &|node| {
        if let Some(call_exp) = node.downcast_ref::<CallExpression>() {
            if let Some(macro_object) = is_macro_call(call_exp, Rc::clone(&env)) {
                let args = quote_args(call_exp);
                let eval_env = extend_macro_env(&macro_object, args);
                let node = eval(macro_object.body.as_node(), Rc::new(RefCell::new(eval_env)));
                if let Some(quote) = node.downcast_ref::<Quote>() {
                    return dyn_clone::clone_box(quote.node.as_ref());
                }
            }
        }
        node
    })
}

fn is_macro_definiation(statement: &dyn Statement) -> bool {
    if let Some(let_statement) = statement.downcast_ref::<LetStatement>() {
        let_statement.value.downcast_ref::<MacroLiteral>().is_some()
    } else {
        false
    }
}

fn add_macro(statement: &dyn Statement, env: Rc<RefCell<Environment>>) {
    if let Some(let_statement) = statement.downcast_ref::<LetStatement>() {
        if let Some(macro_literal) = let_statement.value.downcast_ref::<MacroLiteral>() {
            // 教程里面没有通过 eval 方法，因为默认 eval_to_object 的调用阶段是在求值阶段。我这里只是为了保持统一。
            let macro_object = macro_literal.eval_to_object(Rc::clone(&env));
            env.borrow_mut()
                .set(let_statement.name.string(), macro_object);
        }
    }
}

fn is_macro_call(
    call_expression: &CallExpression,
    env: Rc<RefCell<Environment>>,
) -> Option<Box<Macro>> {
    if let Some(ident) = call_expression.function.downcast_ref::<Identifier>() {
        if let Some(obj) = env.borrow().get(&ident.string()) {
            return obj.downcast::<Macro>().ok();
        }
    }
    None
}

fn quote_args(call_expression: &CallExpression) -> Vec<Quote> {
    let mut args = vec![];
    for arg in call_expression.arguments.iter() {
        args.push(Quote {
            node: dyn_clone::clone_box(arg.as_node()),
        })
    }
    args
}

fn extend_macro_env(macro_object: &Macro, args: Vec<Quote>) -> Environment {
    let mut env = Environment::new_enclosed(Rc::downgrade(&macro_object.env));
    for (i, arg) in args.into_iter().enumerate() {
        env.set(macro_object.parameters[i].string(), Box::new(arg));
    }
    env
}
