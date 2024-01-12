use std::{any::Any, cell::RefMut, mem};

use slotmap::new_key_type;

use crate::{component, reactive::Runtime};

new_key_type! {
    pub struct ItemKey;
}

pub struct Item {
    pub rt: &'static Runtime,
    pub key: ItemKey,
    pub value: Option<Box<dyn Any>>,
    pub action: Option<Box<dyn FnMut()>>,
    pub dependents: Vec<ItemKey>,
}

impl ItemKey {
    pub fn get(self, rt: &Runtime) -> Option<RefMut<Item>> {
        let items = rt.items.borrow_mut();
        RefMut::filter_map(items, |items| items.get_mut(self)).ok()
    }

    pub fn unwrap(self, rt: &Runtime) -> RefMut<Item> {
        self.get(rt).expect("reactive item already dropped")
    }

    pub fn run(self, rt: &Runtime) {
        let mut action = mem::take(&mut self.unwrap(rt).action).expect("item has no action");
        rt.stack.borrow_mut().push(self);
        let prev = rt.tracking.replace(true);
        action();
        rt.tracking.set(prev);
        rt.stack.borrow_mut().pop();
        self.unwrap(rt).action = Some(action);
    }

    pub fn trigger(self, rt: &Runtime) {
        if !rt.tracking.get() {
            return;
        }

        let keys = mem::take(&mut self.unwrap(rt).dependents);
        for key in keys {
            key.run(rt);
        }
    }
}

impl Item {
    pub fn create<F>(f: F) -> (&'static Runtime, ItemKey)
    where
        F: FnOnce(&mut Self),
    {
        let rt = Runtime::unwrap();
        let mut items = rt.items.borrow_mut();
        let key = items.insert_with_key(|key| {
            let mut item = Self {
                rt,
                key,
                value: None,
                action: None,
                dependents: vec![],
            };
            f(&mut item);
            item
        });
        component::with_active(|component| {
            component.add(key);
        });
        (rt, key)
    }

    pub fn track(&mut self) {
        if !self.rt.tracking.get() {
            return;
        }

        let stack = self.rt.stack.borrow();
        if let Some(last) = stack.last() {
            if !self.dependents.contains(last) {
                self.dependents.push(*last);
            }
        }
    }
}
