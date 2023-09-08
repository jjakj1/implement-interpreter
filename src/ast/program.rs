use crate::ast::traits::{Node, Statement};
use std::any::Any;

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
