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
