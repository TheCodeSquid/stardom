use std::{
    any::Any,
    cell::RefCell,
    rc::{Rc, Weak},
};

use super::Runtime;

pub struct Item {
    pub rt: &'static Runtime,
    pub value: Option<RefCell<Box<dyn Any>>>,
    pub action: Option<Box<dyn Fn()>>,
    pub dependents: RefCell<Vec<Weak<Item>>>,
}

impl Item {
    pub fn new() -> Self {
        Self {
            rt: Runtime::unwrap_global(),
            value: None,
            action: None,
            dependents: RefCell::default(),
        }
    }

    pub fn run(self: &Rc<Self>) {
        let action = self.action.as_ref().expect("item has no action");

        self.rt.stack.borrow_mut().push(self.clone());
        action();
        self.rt.stack.borrow_mut().pop();
    }

    pub fn track(&self) {
        let stack = self.rt.stack.borrow();
        if let Some(last) = stack.last() {
            let mut deps = self.dependents.borrow_mut();
            if !deps
                .iter()
                .any(|weak| Weak::ptr_eq(weak, &Rc::downgrade(last)))
            {
                deps.push(Rc::downgrade(last));
            }
        }
    }

    pub fn trigger(&self) {
        let deps = std::mem::take(&mut *self.dependents.borrow_mut());
        for item in deps.iter().filter_map(Weak::upgrade) {
            item.run();
        }
    }
}
