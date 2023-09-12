use crate::environment::Environment;
use crate::object::Object;
use dyn_clone::DynClone;
use std::any::Any;
use std::{cell::RefCell, rc::Rc};

// trait used to upcasting, https://stackoverflow.com/questions/28632968/why-doesnt-rust-support-trait-object-upcasting
pub trait AsNode {
    fn as_node(&self) -> &dyn Node;
}

// TODO: 好像没办法返回一个 Box<dyn Node>
impl<T: Node> AsNode for T {
    fn as_node(&self) -> &dyn Node {
        self
    }
}

pub trait Node: AsNode + DynClone {
    fn token_literal(&self) -> &str;

    // 帮助做 downcast, https://stackoverflow.com/questions/33687447/how-to-get-a-reference-to-a-concrete-type-from-a-trait-object
    // TODO: 好像不能直接在这里提供默认实现返回 self，因为不能在编译器知道 self 的类型？也就不知道类型的生命周期
    // TODO: as_any 不能像 AsNode 一样单独拿出来，如果 T: Node 返回的 any 类型的
    // fn as_any(&self) -> &dyn Any;
    // 这样实现会告诉你 T may not live long enough，因为 Any 的生命周期是 'static。再具体类型中定义没问题的原因是定义的具体类型默认的 lifetime 都是 'static
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

// 表达式
pub trait Expression: Node {
    fn expression_node(&self);
}

dyn_clone::clone_trait_object!(Expression);
