use crate::evaluator::environment::Environment;
use crate::evaluator::object::Object;
use downcast_rs::{impl_downcast, Downcast};
use dyn_clone::DynClone;
use std::{cell::RefCell, rc::Rc};

// trait used to upcasting, https://stackoverflow.com/questions/28632968/why-doesnt-rust-support-trait-object-upcasting
pub trait AsNode {
    fn as_node(&self) -> &dyn Node;

    fn as_boxed_node(self: Box<Self>) -> Box<dyn Node>;

    fn as_mut_node(&mut self) -> &mut dyn Node;
}

impl<T: Node + 'static> AsNode for T {
    fn as_node(&self) -> &dyn Node {
        self
    }

    fn as_boxed_node(self: Box<Self>) -> Box<dyn Node> {
        self
    }

    fn as_mut_node(&mut self) -> &mut dyn Node {
        self
    }
}

pub trait Node: AsNode + DynClone + Downcast {
    fn token_literal(&self) -> &str;

    // 从节点反向打印出本来的代码
    fn string(&self) -> String;

    // 这里还不能使用 &'static mut, 这种引用全局只能有一个，就没法继续传递了
    fn eval_to_object(&self, _environment: Rc<RefCell<Environment>>) -> Box<dyn Object>;
}

impl_downcast!(Node);
dyn_clone::clone_trait_object!(Node);

// 语句
pub trait Statement: Node + Downcast {
    fn statement_node(&self);
}

impl_downcast!(Statement);
dyn_clone::clone_trait_object!(Statement);

// 表达式
pub trait Expression: Node + Downcast {
    fn expression_node(&self);
}

impl_downcast!(Expression);
dyn_clone::clone_trait_object!(Expression);
