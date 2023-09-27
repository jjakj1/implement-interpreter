use crate::ast::expressions::Identifier;
use crate::ast::traits::{Expression, Node, Statement};
use crate::evaluator::environment::Environment;
use crate::evaluator::eval::{eval, eval_block_statement, is_error};
use crate::evaluator::object;
use crate::token::Token;
use std::{cell::RefCell, rc::Rc};

#[derive(Clone)]
pub struct LetStatement {
    pub token: Token,
    pub name: Identifier,
    pub value: Box<dyn Expression>,
}

impl Node for LetStatement {
    fn token_literal(&self) -> &str {
        &self.token.literal
    }

    fn string(&self) -> String {
        let mut out = String::new();
        out.push_str(&format!(
            "{} {} = {};",
            self.token_literal(),
            self.name.string(),
            self.value.string()
        ));
        out
    }

    fn eval_to_object(&self, environment: Rc<RefCell<Environment>>) -> Box<dyn object::Object> {
        let value = eval(self.value.as_node(), environment.clone());
        if is_error(value.as_ref()) {
            return value;
        }
        environment
            .borrow_mut()
            .set(self.name.value.clone(), value)
            .unwrap_or(Box::new(object::Null))
    }
}

impl Statement for LetStatement {
    fn statement_node(&self) {}
}

#[derive(Clone)]
pub struct ReturnStatement {
    pub token: Token,
    pub return_value: Box<dyn Expression>,
}

impl Node for ReturnStatement {
    fn token_literal(&self) -> &str {
        &self.token.literal
    }

    fn string(&self) -> String {
        let mut out = String::new();
        out.push_str(&format!(
            "{} {};",
            self.token_literal(),
            self.return_value.string(),
        ));
        out
    }

    fn eval_to_object(&self, environment: Rc<RefCell<Environment>>) -> Box<dyn object::Object> {
        let value = eval(self.return_value.as_node(), environment);
        if is_error(value.as_ref()) {
            return value;
        }
        Box::new(object::ReturnValue { value })
    }
}

impl Statement for ReturnStatement {
    fn statement_node(&self) {}
}

#[derive(Clone)]
pub struct ExpressionStatement {
    pub token: Token,
    pub expression: Box<dyn Expression>,
}

impl Node for ExpressionStatement {
    fn token_literal(&self) -> &str {
        &self.token.literal
    }

    fn string(&self) -> String {
        self.expression.string()
    }

    fn eval_to_object(&self, environment: Rc<RefCell<Environment>>) -> Box<dyn object::Object> {
        eval(self.expression.as_node(), environment)
    }
}

impl Statement for ExpressionStatement {
    fn statement_node(&self) {}
}

impl Expression for ExpressionStatement {
    fn expression_node(&self) {}
}

#[derive(Clone)]
pub struct BlockStatement {
    pub token: Token, // '{' 词法单元
    pub statements: Vec<Box<dyn Statement>>,
}

impl Node for BlockStatement {
    fn token_literal(&self) -> &str {
        &self.token.literal
    }

    fn string(&self) -> String {
        let mut result = String::new();
        for statement in self.statements.iter() {
            result.push_str(&statement.string())
        }
        result
    }

    fn eval_to_object(&self, environment: Rc<RefCell<Environment>>) -> Box<dyn object::Object> {
        eval_block_statement(self, environment)
    }
}

impl Statement for BlockStatement {
    fn statement_node(&self) {}
}
