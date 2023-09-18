use crate::ast::traits::{Node, Statement};
use crate::evaluator::environment::Environment;
use crate::evaluator::eval::eval_program;
use crate::evaluator::object::Object;
use std::any::Any;
use std::{cell::RefCell, rc::Rc};

#[derive(Clone)]
pub struct Program {
    pub statements: Vec<Box<dyn Statement>>,
}

impl Node for Program {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn token_literal(&self) -> &str {
        self.statements
            .first()
            .map_or("", |statement| statement.token_literal())
    }

    fn string(&self) -> String {
        let mut out = String::new();
        for statement in self.statements.iter() {
            out.push_str(&statement.string())
        }
        out
    }

    fn eval_to_object(&self, environment: Rc<RefCell<Environment>>) -> Option<Box<dyn Object>> {
        eval_program(self, environment)
    }
}
