use crate::evaluator::environment::Environment;
use crate::evaluator::object::Object;
use dyn_clone::DynClone;
use std::any::Any;
use std::{cell::RefCell, rc::Rc};

// trait used to upcasting, https://stackoverflow.com/questions/28632968/why-doesnt-rust-support-trait-object-upcasting
pub trait AsNode {
    fn as_node(&self) -> &dyn Node;
}

impl<T: Node> AsNode for T {
    fn as_node(&self) -> &dyn Node {
        self
    }
}

pub trait Node: AsNode + DynClone + Any {
    fn token_literal(&self) -> &str;

    // 帮助做 downcast, https://stackoverflow.com/questions/33687447/how-to-get-a-reference-to-a-concrete-type-from-a-trait-object
    // 不能直接在这里提供默认实现返回 self，因为不能在编译器知道 self 的类型？不知道 self 有没有实现 Any
    fn as_any(&self) -> &dyn Any;

    // 从节点反向打印出本来的代码
    fn string(&self) -> String;

    // 这里还不能使用 &'static mut, 这种引用全局只能有一个，就没法继续传递了
    fn eval_to_object(&self, _environment: Rc<RefCell<Environment>>) -> Option<Box<dyn Object>> {
        None
    }
}

dyn_clone::clone_trait_object!(Node);

// 语句
pub trait Statement: Node {
    fn statement_node(&self);
}

dyn_clone::clone_trait_object!(Statement);

pub struct HashKey {}

// 表达式
pub trait Expression: Node {
    fn expression_node(&self);
}

dyn_clone::clone_trait_object!(Expression);
