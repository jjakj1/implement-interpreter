use super::traits::AsNode;
use crate::ast::statements::BlockStatement;
use crate::ast::traits::{Expression, Node};
use crate::evaluator::environment::Environment;
use crate::evaluator::eval::{
    apply_function, eval, eval_expressions, eval_hash_literal, eval_identifier,
    eval_index_expression, eval_infix_expression, eval_prefix_expression, is_error, is_truthy,
};
use crate::evaluator::object::{self, Array, Function, Macro, StringObject};
use crate::quote::quote;
use crate::token::Token;
use by_address::ByAddress;
use std::collections::HashMap;
use std::{cell::RefCell, rc::Rc};

// 标识符
#[derive(Clone)]
pub struct Identifier {
    pub token: Token,
    pub value: String,
}

impl Node for Identifier {
    fn token_literal(&self) -> &str {
        &self.token.literal
    }

    fn string(&self) -> String {
        self.value.clone()
    }

    fn eval_to_object(&self, environment: Rc<RefCell<Environment>>) -> Box<dyn object::Object> {
        eval_identifier(self, environment)
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
    fn token_literal(&self) -> &str {
        &self.token.literal
    }

    fn string(&self) -> String {
        self.value.to_string()
    }

    fn eval_to_object(&self, _environment: Rc<RefCell<Environment>>) -> Box<dyn object::Object> {
        Box::new(object::Integer { value: self.value })
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
    fn token_literal(&self) -> &str {
        &self.token.literal
    }

    fn string(&self) -> String {
        self.value.to_string()
    }

    fn eval_to_object(&self, _environment: Rc<RefCell<Environment>>) -> Box<dyn object::Object> {
        if self.value {
            Box::new(object::Boolean::True)
        } else {
            Box::new(object::Boolean::False)
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

    fn eval_to_object(&self, environment: Rc<RefCell<Environment>>) -> Box<dyn object::Object> {
        let condition = eval(self.condition.as_node(), environment.clone());
        if is_error(condition.as_ref()) {
            return condition;
        }

        if is_truthy(condition.as_ref()) {
            eval(self.consequence.as_node(), environment)
        } else if let Some(alternative) = &self.alternative {
            eval(alternative.as_node(), environment)
        } else {
            Box::new(object::Null)
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

    fn eval_to_object(&self, environment: Rc<RefCell<Environment>>) -> Box<dyn object::Object> {
        Box::new(Function {
            parameters: self.parameters.clone(),
            body: self.body.clone(),
            env: environment,
        })
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

    fn eval_to_object(&self, environment: Rc<RefCell<Environment>>) -> Box<dyn object::Object> {
        if self.function.token_literal() == "quote" {
            if let Some(first) = self.arguments.first() {
                let first = dyn_clone::clone_box(first.as_ref());
                return quote(&mut first.as_boxed_node(), Rc::clone(&environment));
            } else {
                return Box::new(object::Error {
                    message: "`quote` needs to be called with one argument".to_owned(),
                });
            }
        }
        let func = eval(self.function.as_node(), environment.clone());
        if is_error(func.as_ref()) {
            return func;
        }
        let mut params = eval_expressions(&self.arguments, environment);
        if params.len() == 1 && is_error(params.first().unwrap().as_ref()) {
            return params.swap_remove(0);
        }
        apply_function(func.as_ref(), &params)
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
    fn token_literal(&self) -> &str {
        &self.token.literal
    }

    fn string(&self) -> String {
        format!("({}{})", self.operator, self.right.string())
    }

    fn eval_to_object(&self, environment: Rc<RefCell<Environment>>) -> Box<dyn object::Object> {
        let right = eval(self.right.as_node(), environment);
        if is_error(right.as_ref()) {
            return right;
        }
        eval_prefix_expression(&self.operator, right.as_ref())
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

    fn eval_to_object(&self, environment: Rc<RefCell<Environment>>) -> Box<dyn object::Object> {
        let left = eval(self.left.as_node(), environment.clone());
        if is_error(left.as_ref()) {
            return left;
        }
        let right = eval(self.right.as_node(), environment);
        if is_error(right.as_ref()) {
            return right;
        }
        eval_infix_expression(left.as_ref(), &self.operator, right.as_ref())
    }
}

impl Expression for InfixExpression {
    fn expression_node(&self) {}
}

#[derive(Clone)]
pub struct StringLiteral {
    pub token: Token,
    pub value: String,
}

impl Node for StringLiteral {
    fn string(&self) -> String {
        self.value.clone()
    }

    fn token_literal(&self) -> &str {
        &self.token.literal
    }

    fn eval_to_object(&self, _environment: Rc<RefCell<Environment>>) -> Box<dyn object::Object> {
        Box::new(StringObject {
            value: self.value.clone(),
        })
    }
}

impl Expression for StringLiteral {
    fn expression_node(&self) {}
}

#[derive(Clone)]
pub struct ArrayLiteral {
    pub token: Token, // [ 词法单元
    pub elements: Vec<Box<dyn Expression>>,
}

impl Node for ArrayLiteral {
    fn string(&self) -> String {
        let elements = self
            .elements
            .iter()
            .map(|element| element.string())
            .collect::<Vec<_>>()
            .join(", ");
        format!("[{}]", elements)
    }

    fn token_literal(&self) -> &str {
        &self.token.literal
    }

    fn eval_to_object(&self, environment: Rc<RefCell<Environment>>) -> Box<dyn object::Object> {
        let elements = eval_expressions(&self.elements, environment);
        if elements.len() == 1
            && matches!(
                elements.get(0).unwrap().object_type(),
                object::ObjectType::Error
            )
        {
            return dyn_clone::clone_box(elements[0].as_ref());
        }

        Box::new(Array { elements })
    }
}

impl Expression for ArrayLiteral {
    fn expression_node(&self) {}
}

#[derive(Clone)]
pub struct IndexExpression {
    pub token: Token,
    pub left: Box<dyn Expression>,
    pub index: Box<dyn Expression>,
}

impl Node for IndexExpression {
    fn string(&self) -> String {
        format!("({}[{}])", self.left.string(), self.index.string())
    }

    fn token_literal(&self) -> &str {
        &self.token.literal
    }

    fn eval_to_object(&self, environment: Rc<RefCell<Environment>>) -> Box<dyn object::Object> {
        let left = eval(self.left.as_node(), Rc::clone(&environment));
        if is_error(left.as_ref()) {
            return left;
        }
        let index = eval(self.index.as_node(), environment);
        if is_error(index.as_ref()) {
            return index;
        }
        eval_index_expression(left.as_ref(), index.as_ref())
    }
}

impl Expression for IndexExpression {
    fn expression_node(&self) {}
}

#[derive(Clone)]
pub struct HashLiteral {
    pub token: Token,
    pub pairs: HashMap<ByAddress<Box<dyn Expression>>, Box<dyn Expression>>,
}

impl Node for HashLiteral {
    fn string(&self) -> String {
        let key_values = self
            .pairs
            .iter()
            .map(|(key, value)| format!("{}: {}", key.string(), value.string()))
            .collect::<Vec<_>>()
            .join(", ");
        format!("{{{}}}", key_values)
    }

    fn token_literal(&self) -> &str {
        &self.token.literal
    }

    fn eval_to_object(&self, environment: Rc<RefCell<Environment>>) -> Box<dyn object::Object> {
        eval_hash_literal(self, environment)
    }
}

impl Expression for HashLiteral {
    fn expression_node(&self) {}
}

#[derive(Clone)]
pub struct MacroLiteral {
    pub token: Token,
    pub parameters: Vec<Identifier>,
    pub body: BlockStatement,
}

impl Node for MacroLiteral {
    fn token_literal(&self) -> &str {
        &self.token.literal
    }

    fn string(&self) -> String {
        let params = self
            .parameters
            .iter()
            .map(|ident| ident.string())
            .collect::<Vec<_>>()
            .join(", ");
        format!("{}({}){}", self.token_literal(), params, self.body.string())
    }

    fn eval_to_object(&self, environment: Rc<RefCell<Environment>>) -> Box<dyn object::Object> {
        Box::new(Macro {
            parameters: self.parameters.clone(),
            body: self.body.clone(),
            env: environment,
        })
    }
}

impl Expression for MacroLiteral {
    fn expression_node(&self) {}
}
