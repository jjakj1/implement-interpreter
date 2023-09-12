use crate::ast::statements::BlockStatement;
use crate::ast::traits::{Expression, Node};
use crate::environment::Environment;
use crate::evaluator::{
    apply_function, eval, eval_expressions, eval_infix_expression, eval_prefix_expression,
    is_error, is_truthy,
};
use crate::object::{self, Function};
use crate::token::Token;
use std::any::Any;
use std::{cell::RefCell, rc::Rc};

use super::traits::AsNode;

// 标识符
#[derive(Clone)]
pub struct Identifier {
    pub token: Token,
    pub value: String,
}

impl Node for Identifier {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn token_literal(&self) -> &str {
        &self.token.literal
    }

    fn string(&self) -> String {
        self.value.clone()
    }

    fn eval_to_object(
        &self,
        environment: Rc<RefCell<Environment>>,
    ) -> Option<Box<dyn object::Object>> {
        environment
            .borrow()
            .get(&self.value)
            .or(Some(Box::new(object::Error {
                message: format!("identifier not found: {}", self.value),
            })))
    }
}

impl Expression for Identifier {
    fn expression_node(&self) {}
}

// 整数字面量
#[derive(Clone)]
pub struct IntegerLiteral {
    pub token: Token,
    pub value: i64,
}

impl Node for IntegerLiteral {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn token_literal(&self) -> &str {
        &self.token.literal
    }

    fn string(&self) -> String {
        self.value.to_string()
    }

    fn eval_to_object(
        &self,
        _environment: Rc<RefCell<Environment>>,
    ) -> Option<Box<dyn object::Object>> {
        Some(Box::new(object::Integer { value: self.value }))
    }
}

impl Expression for IntegerLiteral {
    fn expression_node(&self) {}
}

#[derive(Clone)]
pub struct Boolean {
    pub token: Token,
    pub value: bool,
}

impl Node for Boolean {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn token_literal(&self) -> &str {
        &self.token.literal
    }

    fn string(&self) -> String {
        self.value.to_string()
    }

    fn eval_to_object(
        &self,
        _environment: Rc<RefCell<Environment>>,
    ) -> Option<Box<dyn object::Object>> {
        if self.value {
            Some(Box::new(object::Boolean::True))
        } else {
            Some(Box::new(object::Boolean::False))
        }
    }
}

impl Expression for Boolean {
    fn expression_node(&self) {}
}

#[derive(Clone)]
pub struct IfExpression {
    pub token: Token,
    pub condition: Box<dyn Expression>,
    pub consequence: BlockStatement,
    pub alternative: Option<BlockStatement>,
}

impl Node for IfExpression {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn token_literal(&self) -> &str {
        &self.token.literal
    }

    fn string(&self) -> String {
        let mut result = format!(
            "{} {} {}",
            self.token_literal(),
            self.condition.string(),
            self.consequence.string()
        );
        if let Some(alternative) = self.alternative.as_ref() {
            result.push_str(&format!("else {}", alternative.string()))
        }
        result
    }

    fn eval_to_object(
        &self,
        environment: Rc<RefCell<Environment>>,
    ) -> Option<Box<dyn object::Object>> {
        let condition = eval(self.condition.as_node(), environment.clone());
        if is_error(&condition) {
            return condition;
        }

        if is_truthy(condition)? {
            eval(self.consequence.as_node(), environment)
        } else if let Some(alternative) = &self.alternative {
            eval(alternative.as_node(), environment)
        } else {
            Some(Box::new(object::Null))
        }
    }
}

impl Expression for IfExpression {
    fn expression_node(&self) {}
}

#[derive(Clone)]
pub struct FunctionLiteral {
    pub token: Token,
    pub parameters: Vec<Identifier>, // 这里是一个函数定义，因此只能是 Identifier
    pub body: BlockStatement,
}

impl Node for FunctionLiteral {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn token_literal(&self) -> &str {
        &self.token.literal
    }

    fn string(&self) -> String {
        let parameters = self
            .parameters
            .iter()
            .map(|expression| expression.string())
            .collect::<Vec<_>>()
            .join(", ");
        format!(
            "{}({}) {}",
            self.token_literal(),
            parameters,
            self.body.string()
        )
    }

    fn eval_to_object(
        &self,
        environment: Rc<RefCell<Environment>>,
    ) -> Option<Box<dyn object::Object>> {
        Some(Box::new(Function {
            parameters: self.parameters.clone(),
            body: self.body.clone(),
            env: environment,
        }))
    }
}

impl Expression for FunctionLiteral {
    fn expression_node(&self) {}
}

#[derive(Clone)]
pub struct CallExpression {
    pub token: Token, // '(' 词法单元
    pub function: Box<dyn Expression>,
    pub arguments: Vec<Box<dyn Expression>>,
}

impl Node for CallExpression {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn token_literal(&self) -> &str {
        &self.token.literal
    }

    fn string(&self) -> String {
        let args = self
            .arguments
            .iter()
            .map(|arg| arg.string())
            .collect::<Vec<_>>()
            .join(", ");
        format!("{}({})", self.function.string(), args)
    }

    fn eval_to_object(
        &self,
        environment: Rc<RefCell<Environment>>,
    ) -> Option<Box<dyn object::Object>> {
        let func = eval(self.function.as_node(), environment.clone());
        if is_error(&func) {
            return func;
        }
        let params = eval_expressions(&self.arguments, environment)?;
        apply_function(func?, params)
    }
}

impl Expression for CallExpression {
    fn expression_node(&self) {}
}

#[derive(Clone)]
pub struct PrefixExpression {
    pub token: Token, // 前置的 token
    pub operator: String,
    pub right: Box<dyn Expression>,
}

impl Node for PrefixExpression {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn token_literal(&self) -> &str {
        &self.token.literal
    }

    fn string(&self) -> String {
        format!("({}{})", self.operator, self.right.string())
    }

    fn eval_to_object(
        &self,
        environment: Rc<RefCell<Environment>>,
    ) -> Option<Box<dyn object::Object>> {
        let right = eval(self.right.as_node(), environment);
        if is_error(&right) {
            return right;
        }
        eval_prefix_expression(&self.operator, right)
    }
}

impl Expression for PrefixExpression {
    fn expression_node(&self) {}
}

#[derive(Clone)]
pub struct InfixExpression {
    pub token: Token, // 中间的 token
    pub left: Box<dyn Expression>,
    pub operator: String,
    pub right: Box<dyn Expression>,
}

impl Node for InfixExpression {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn token_literal(&self) -> &str {
        &self.token.literal
    }

    fn string(&self) -> String {
        format!(
            "({} {} {})",
            self.left.string(),
            self.operator,
            self.right.string()
        )
    }

    fn eval_to_object(
        &self,
        environment: Rc<RefCell<Environment>>,
    ) -> Option<Box<dyn object::Object>> {
        let left = eval(self.left.as_node(), environment.clone());
        if is_error(&left) {
            return left;
        }
        let right = eval(self.right.as_node(), environment);
        if is_error(&right) {
            return right;
        }
        eval_infix_expression(left, &self.operator, right)
    }
}

impl Expression for InfixExpression {
    fn expression_node(&self) {}
}
