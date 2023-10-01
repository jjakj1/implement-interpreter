use downcast_rs::{impl_downcast, Downcast};
use dyn_clone::DynClone;
use once_cell::sync::Lazy;
use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::hash::Hasher;
use std::{cell::RefCell, rc::Rc, sync::Arc};

use super::environment::Environment;
use crate::ast::{expressions::Identifier, statements::BlockStatement, traits::Node};

type BuiltinFunction = dyn Fn(&[&dyn Object]) -> Box<dyn Object> + Send + Sync + 'static;

pub static BUILTINS: Lazy<HashMap<&'static str, Builtin>> = Lazy::new(|| {
    HashMap::from([
        (
            "len",
            Builtin {
                func: Arc::new(object_len),
            },
        ),
        (
            "first",
            Builtin {
                func: Arc::new(array_first),
            },
        ),
        (
            "last",
            Builtin {
                func: Arc::new(array_last),
            },
        ),
        (
            "rest",
            Builtin {
                func: Arc::new(array_rest),
            },
        ),
        (
            "push",
            Builtin {
                func: Arc::new(array_push),
            },
        ),
        (
            "puts",
            Builtin {
                func: Arc::new(puts),
            },
        ),
    ])
});

fn object_len(objects: &[&dyn Object]) -> Box<dyn Object> {
    if objects.len() != 1 {
        return Box::new(Error {
            message: format!("wrong number of arguments: got={}, want=1", objects.len()),
        });
    }

    let first = *objects.first().unwrap();

    match first.object_type() {
        ObjectType::String => {
            let string = first.downcast_ref::<StringObject>().unwrap();
            Box::new(Integer {
                value: string.value.len() as i64,
            })
        }
        ObjectType::Array => {
            let array = first.downcast_ref::<Array>().unwrap();
            Box::new(Integer {
                value: array.elements.len() as i64,
            })
        }
        _ => Box::new(Error {
            message: format!(
                "argument to `len` not supported, got {:?}",
                first.object_type()
            ),
        }),
    }
}

fn array_first(objects: &[&dyn Object]) -> Box<dyn Object> {
    if objects.len() != 1 {
        return Box::new(Error {
            message: format!("wrong number of arguments: got={}, want=1", objects.len()),
        });
    }

    let first = *objects.first().unwrap();

    match first.object_type() {
        ObjectType::Array => {
            let array = first.downcast_ref::<Array>().unwrap();
            array
                .elements
                .first()
                .map_or(Box::new(Null), |first| dyn_clone::clone_box(first.as_ref()))
        }
        _ => Box::new(Error {
            message: format!(
                "argument to `first` must be Array, got {:?}",
                first.object_type()
            ),
        }),
    }
}

fn array_last(objects: &[&dyn Object]) -> Box<dyn Object> {
    if objects.len() != 1 {
        return Box::new(Error {
            message: format!("wrong number of arguments: got={}, want=1", objects.len()),
        });
    }

    let first = *objects.first().unwrap();

    match first.object_type() {
        ObjectType::Array => {
            let array = first.downcast_ref::<Array>().unwrap();
            array
                .elements
                .iter()
                .last()
                .map_or(Box::new(Null), |last| dyn_clone::clone_box(last.as_ref()))
        }
        _ => Box::new(Error {
            message: format!(
                "argument to `last` must be Array, got {:?}",
                first.object_type()
            ),
        }),
    }
}

fn array_rest(objects: &[&dyn Object]) -> Box<dyn Object> {
    if objects.len() != 1 {
        return Box::new(Error {
            message: format!("wrong number of arguments: got={}, want=1", objects.len()),
        });
    }

    let first = *objects.first().unwrap();

    match first.object_type() {
        ObjectType::Array => {
            let array = dyn_clone::clone_box(first)
                .downcast::<Array>()
                .map_err(|_| "Shouldn't happen.")
                .unwrap();
            Box::new(Array {
                elements: array.elements.into_iter().skip(1).collect::<Vec<_>>(),
            })
        }
        _ => Box::new(Error {
            message: format!(
                "argument to `last` must be Array, got {:?}",
                first.object_type()
            ),
        }),
    }
}

fn array_push(objects: &[&dyn Object]) -> Box<dyn Object> {
    if objects.len() != 2 {
        return Box::new(Error {
            message: format!("wrong number of arguments: got={}, want=2", objects.len()),
        });
    }

    let first = *objects.first().unwrap();
    let object = dyn_clone::clone_box(*objects.get(1).unwrap());

    match first.object_type() {
        ObjectType::Array => {
            let mut array = dyn_clone::clone_box(first)
                .downcast::<Array>()
                .map_err(|_| "Shouldn't happen.")
                .unwrap();
            array.elements.push(object);
            array
        }
        _ => Box::new(Error {
            message: format!(
                "argument to `push` must be Array, got {:?}",
                first.object_type()
            ),
        }),
    }
}

fn puts(objects: &[&dyn Object]) -> Box<dyn Object> {
    for &object in objects {
        println!("{}", object.inspect());
    }
    Box::new(Null)
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ObjectType {
    Integer,
    Boolean,
    Null,
    ReturnValue,
    Error,
    Function,
    String,
    Builtin,
    Array,
    Hash,
    Quote,
    Macro,
}

pub trait Object: DynClone + Downcast {
    fn object_type(&self) -> ObjectType;

    fn inspect(&self) -> String;
}

impl_downcast!(Object);
dyn_clone::clone_trait_object!(Object);

pub trait Hashable {
    fn hash_key(&self) -> HashKey;
}

#[derive(Clone)]
pub struct Integer {
    pub value: i64,
}

impl Hashable for Integer {
    fn hash_key(&self) -> HashKey {
        HashKey {
            object_type: self.object_type(),
            value: self.value as u64,
        }
    }
}

impl Object for Integer {
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

impl Hashable for Boolean {
    fn hash_key(&self) -> HashKey {
        let value = if matches!(self, Boolean::True) { 1 } else { 0 };
        HashKey {
            object_type: self.object_type(),
            value,
        }
    }
}

impl Object for Boolean {
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

#[derive(Clone)]
pub struct StringObject {
    pub value: String,
}

impl Hashable for StringObject {
    fn hash_key(&self) -> HashKey {
        let mut hasher = DefaultHasher::new();
        hasher.write(self.value.as_bytes());
        HashKey {
            object_type: self.object_type(),
            value: hasher.finish(),
        }
    }
}

impl Object for StringObject {
    fn inspect(&self) -> String {
        self.value.clone()
    }

    fn object_type(&self) -> ObjectType {
        ObjectType::String
    }
}

#[derive(Clone)]
pub struct Builtin {
    pub func: Arc<BuiltinFunction>,
}

impl Object for Builtin {
    fn inspect(&self) -> String {
        "builtin function".to_owned()
    }

    fn object_type(&self) -> ObjectType {
        ObjectType::Builtin
    }
}

#[derive(Clone)]
pub struct Array {
    pub elements: Vec<Box<dyn Object>>,
}

impl Object for Array {
    fn inspect(&self) -> String {
        let elements = self
            .elements
            .iter()
            .map(|element| element.inspect())
            .collect::<Vec<_>>()
            .join(", ");
        format!("[{}]", elements)
    }

    fn object_type(&self) -> ObjectType {
        ObjectType::Array
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct HashKey {
    object_type: ObjectType,
    value: u64,
}

#[derive(Clone)]
pub struct HashPair {
    pub key: Box<dyn Object>,
    pub value: Box<dyn Object>,
}

#[derive(Clone)]
pub struct Hash {
    pub pairs: HashMap<HashKey, HashPair>,
}

impl Object for Hash {
    fn inspect(&self) -> String {
        let pairs = self
            .pairs
            .values()
            .map(|pair| format!("{}: {}", pair.key.inspect(), pair.value.inspect()))
            .collect::<Vec<_>>()
            .join(", ");
        format!("{{{}}}", pairs)
    }

    fn object_type(&self) -> ObjectType {
        ObjectType::Hash
    }
}

#[derive(Clone)]
pub struct Quote {
    pub node: Box<dyn Node>,
}

impl Object for Quote {
    fn object_type(&self) -> ObjectType {
        ObjectType::Quote
    }

    fn inspect(&self) -> String {
        format!("QUOTE({})", self.node.string())
    }
}

#[derive(Clone)]
pub struct Macro {
    pub parameters: Vec<Identifier>,
    pub body: BlockStatement,
    pub env: Rc<RefCell<Environment>>,
}

impl Object for Macro {
    fn inspect(&self) -> String {
        let params = self
            .parameters
            .iter()
            .map(|p| p.string())
            .collect::<Vec<_>>()
            .join(", ");
        format!("macro ({}) {{\n{}\n}}", params, self.body.string())
    }

    fn object_type(&self) -> ObjectType {
        ObjectType::Macro
    }
}
