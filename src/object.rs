use dyn_clone::DynClone;
use std::any::Any;
use std::{cell::RefCell, rc::Rc};

use crate::{
    ast::{expressions::Identifier, statements::BlockStatement, traits::Node},
    environment::Environment,
};

#[derive(Debug, PartialEq)]
pub enum ObjectType {
    Integer,
    Boolean,
    Null,
    ReturnValue,
    Error,
    Function,
}

pub trait Object: DynClone {
    // https://stackoverflow.com/questions/31949579/understanding-and-relationship-between-box-ref-and
    // 可以看 std::any::Any 的文档，只对 Box<Self> 有 downcast 方法。而对于 downcast_ref 没有。
    // 上面的解释说明了实际上 Box 也是拥有数据的 ownership，而 & 只是 borrow。因此我理解不能直接对引用的数据做转换
    fn as_any(self: Box<Self>) -> Box<dyn Any>;

    fn object_type(&self) -> ObjectType;

    fn inspect(&self) -> String;
}

dyn_clone::clone_trait_object!(Object);

#[derive(Clone)]
pub struct Integer {
    pub value: i64,
}

impl Object for Integer {
    fn as_any(self: Box<Self>) -> Box<dyn Any> {
        self
    }

    fn inspect(&self) -> String {
        format!("{}", self.value)
    }

    fn object_type(&self) -> ObjectType {
        ObjectType::Integer
    }
}

#[derive(PartialEq, Eq, Clone)]
pub enum Boolean {
    True,
    False,
}

impl Object for Boolean {
    fn as_any(self: Box<Self>) -> Box<dyn Any> {
        self
    }

    fn inspect(&self) -> String {
        match self {
            Boolean::True => "true".to_owned(),
            Boolean::False => "false".to_owned(),
        }
    }

    fn object_type(&self) -> ObjectType {
        ObjectType::Boolean
    }
}

impl Boolean {
    pub fn value(&self) -> bool {
        match self {
            Boolean::True => true,
            Boolean::False => false,
        }
    }

    pub fn from_native_bool(input: bool) -> Self {
        if input {
            Boolean::True
        } else {
            Boolean::False
        }
    }
}

#[derive(Clone)]
pub struct Null;

impl Object for Null {
    fn as_any(self: Box<Self>) -> Box<dyn Any> {
        self
    }

    fn inspect(&self) -> String {
        "null".to_owned()
    }

    fn object_type(&self) -> ObjectType {
        ObjectType::Null
    }
}

#[derive(Clone)]
pub struct ReturnValue {
    pub value: Box<dyn Object>,
}

impl Object for ReturnValue {
    fn as_any(self: Box<Self>) -> Box<dyn Any> {
        self
    }

    fn inspect(&self) -> String {
        self.value.inspect()
    }

    fn object_type(&self) -> ObjectType {
        ObjectType::ReturnValue
    }
}

#[derive(Clone)]
pub struct Error {
    pub message: String,
}

impl Object for Error {
    fn as_any(self: Box<Self>) -> Box<dyn Any> {
        self
    }

    fn inspect(&self) -> String {
        format!("Error: {}", self.message)
    }

    fn object_type(&self) -> ObjectType {
        ObjectType::Error
    }
}

#[derive(Clone)]
pub struct Function {
    pub parameters: Vec<Identifier>,
    pub body: BlockStatement,
    pub env: Rc<RefCell<Environment>>,
}

impl Object for Function {
    fn as_any(self: Box<Self>) -> Box<dyn Any> {
        self
    }

    fn inspect(&self) -> String {
        let params = self
            .parameters
            .iter()
            .map(|p| p.string())
            .collect::<Vec<_>>()
            .join(", ");
        format!("fn ({}) {{\n{}\n}}", params, self.body.string())
    }

    fn object_type(&self) -> ObjectType {
        ObjectType::Function
    }
}
