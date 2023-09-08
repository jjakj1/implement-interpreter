use crate::ast::expressions::Identifier;
use crate::ast::traits::{Expression, Node, Statement};
use crate::token::Token;
use std::any::Any;

pub struct LetStatement {
    pub token: Token,
    pub name: Identifier,
    pub value: Box<dyn Expression>,
}

impl Node for LetStatement {
    fn token_literal(&self) -> &str {
        &self.token.literal
    }

    fn as_any(&self) -> &dyn Any {
        self
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
}

impl Statement for LetStatement {
    fn statement_node(&self) {}
}

pub struct ReturnStatement {
    pub token: Token,
    pub return_value: Box<dyn Expression>,
}

impl Node for ReturnStatement {
    fn token_literal(&self) -> &str {
        &self.token.literal
    }

    fn as_any(&self) -> &dyn Any {
        self
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
}

impl Statement for ReturnStatement {
    fn statement_node(&self) {}
}

pub struct ExpressionStatement {
    pub token: Token,
    pub expression: Box<dyn Expression>,
}

impl Node for ExpressionStatement {
    fn token_literal(&self) -> &str {
        &self.token.literal
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn string(&self) -> String {
        self.expression.string()
    }
}

impl Statement for ExpressionStatement {
    fn statement_node(&self) {}
}

impl Expression for ExpressionStatement {
    fn expression_node(&self) {}
}

pub struct BlockStatement {
    pub token: Token, // '{' 词法单元
    pub statements: Vec<Box<dyn Statement>>,
}

impl Node for BlockStatement {
    fn token_literal(&self) -> &str {
        &self.token.literal
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn string(&self) -> String {
        let mut result = String::new();
        for statement in self.statements.iter() {
            result.push_str(&statement.string())
        }
        result
    }
}

impl Statement for BlockStatement {
    fn statement_node(&self) {}
}
