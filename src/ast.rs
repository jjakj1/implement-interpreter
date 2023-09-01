use crate::token::Token;
use std::any::Any;

pub trait Node {
    fn token_literal(&self) -> &str;

    // 帮助做 downcast, https://stackoverflow.com/questions/33687447/how-to-get-a-reference-to-a-concrete-type-from-a-trait-object
    fn as_any(&self) -> &dyn Any;

    // 从节点反向打印出本来的代码
    fn string(&self) -> String;
}

// 语句
pub trait Statement: Node {
    fn statement_node(&self);
}

// 表达式
pub trait Expression: Node {
    fn expression_node(&self);
}

// 根节点
pub struct Program {
    pub statements: Vec<Box<dyn Statement>>,
}

impl Node for Program {
    fn token_literal(&self) -> &str {
        self.statements
            .first()
            .map_or("", |statement| statement.token_literal())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn string(&self) -> String {
        let mut out = String::new();
        for statement in self.statements.iter() {
            out.push_str(&statement.string())
        }
        out
    }
}

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

// TODO: 这个地方我想写成 pub struct<E: Expression> LetStatement<E> 但好像在 parse 里面使用这个的时候，Expression 都确定不了具体类型
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

#[cfg(test)]
mod tests {
    use super::{Expression, Identifier, LetStatement, Program, Statement};
    use crate::{
        ast::Node,
        token::{Token, TokenType},
    };

    #[test]
    fn test_string() {
        let program = Program {
            statements: vec![Box::new(LetStatement {
                token: Token {
                    token_type: TokenType::Let,
                    literal: "let".to_owned(),
                },
                name: Identifier {
                    token: Token {
                        token_type: TokenType::Ident,
                        literal: "myVar".to_owned(),
                    },
                    value: "myVar".to_owned(),
                },
                value: Box::new(Identifier {
                    token: Token {
                        token_type: TokenType::Ident,
                        literal: "anotherVar".to_owned(),
                    },
                    value: "anotherVar".to_owned(),
                }) as Box<dyn Expression>,
            }) as Box<dyn Statement>],
        };

        assert_eq!(program.string(), "let myVar = anotherVar;");
    }
}
