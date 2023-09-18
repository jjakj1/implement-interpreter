use super::object;
use std::collections::HashMap;
use std::{cell::RefCell, rc::Weak};

pub struct Environment {
    store: HashMap<String, Box<dyn object::Object>>,
    outer: Weak<RefCell<Environment>>,
}

impl Environment {
    pub fn new() -> Self {
        Environment {
            store: HashMap::new(),
            outer: Weak::new(),
        }
    }

    pub fn new_enclosed(outer: Weak<RefCell<Environment>>) -> Self {
        Environment {
            store: HashMap::new(),
            outer,
        }
    }

    pub fn get(&self, name: &str) -> Option<Box<dyn object::Object>> {
        self.store
            .get(name)
            .map(|boxed_object| dyn_clone::clone_box(&**boxed_object))
            .or_else(|| self.outer.upgrade().and_then(|env| env.borrow().get(name)))
    }

    pub fn set(
        &mut self,
        name: String,
        value: Box<dyn object::Object>,
    ) -> Option<Box<dyn object::Object>> {
        self.store.insert(name, value)
    }
}

impl Default for Environment {
    fn default() -> Self {
        Environment::new()
    }
}
