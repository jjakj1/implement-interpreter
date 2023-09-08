use crate::ast::statements::BlockStatement;
use crate::ast::traits::{Expression, Node};
use crate::token::Token;
use std::any::Any;

// 标识符
pub struct Identifier {
    pub token: Token,
    pub value: String,
}

impl Node for Identifier {
    fn token_literal(&self) -> &str {
        &self.token.literal
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn string(&self) -> String {
        self.value.clone()
    }
}

impl Expression for Identifier {
    fn expression_node(&self) {}
}

// 整数字面量
pub struct IntegerLiteral {
    pub token: Token,
    pub value: i64,
}

impl Node for IntegerLiteral {
    fn token_literal(&self) -> &str {
        &self.token.literal
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn string(&self) -> String {
        self.value.to_string()
    }
}

impl Expression for IntegerLiteral {
    fn expression_node(&self) {}
}

pub struct Boolean {
    pub token: Token,
    pub value: bool,
}

impl Node for Boolean {
    fn token_literal(&self) -> &str {
        &self.token.literal
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn string(&self) -> String {
        self.value.to_string()
    }
}

impl Expression for Boolean {
    fn expression_node(&self) {}
}

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

    fn as_any(&self) -> &dyn Any {
        self
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
}

impl Expression for IfExpression {
    fn expression_node(&self) {}
}

pub struct FunctionLiteral {
    pub token: Token,
    pub parameters: Vec<Identifier>, // 这里是一个函数定义，因此只能是 Identifier
    pub body: BlockStatement,
}

impl Node for FunctionLiteral {
    fn token_literal(&self) -> &str {
        &self.token.literal
    }

    fn as_any(&self) -> &dyn Any {
        self
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
}

impl Expression for FunctionLiteral {
    fn expression_node(&self) {}
}

pub struct CallExpression {
    pub token: Token, // '(' 词法单元
    pub function: Box<dyn Expression>,
    pub arguments: Vec<Box<dyn Expression>>,
}

impl Node for CallExpression {
    fn token_literal(&self) -> &str {
        &self.token.literal
    }

    fn as_any(&self) -> &dyn Any {
        self
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
}

impl Expression for CallExpression {
    fn expression_node(&self) {}
}

pub struct PrefixExpression {
    pub token: Token, // 前置的 token
    pub operator: String,
    pub right: Box<dyn Expression>,
}

impl Node for PrefixExpression {
    fn token_literal(&self) -> &str {
        &self.token.literal
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn string(&self) -> String {
        format!("({}{})", self.operator, self.right.string())
    }
}

impl Expression for PrefixExpression {
    fn expression_node(&self) {}
}

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

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn string(&self) -> String {
        format!(
            "({} {} {})",
            self.left.string(),
            self.operator,
            self.right.string()
        )
    }
}

impl Expression for InfixExpression {
    fn expression_node(&self) {}
}
