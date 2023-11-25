use std::{
    any::Any,
    cell::{Ref, RefMut},
    mem,
};

use slotmap::new_key_type;

use crate::Runtime;

new_key_type! {
    pub struct ItemKey;
}

#[derive(Default)]
pub(crate) struct Item {
    pub value: Option<Box<dyn Any>>,
    pub action: Option<Box<dyn FnMut()>>,

    pub parent: Option<ItemKey>,
    pub dependents: Vec<ItemKey>,
}

impl ItemKey {
    pub(crate) fn get(&self, rt: &'static Runtime) -> Ref<Item> {
        let items = rt.items.borrow();
        Ref::map(items, |items| {
            items.get(*self).expect("item used after internal drop")
        })
    }

    pub(crate) fn get_mut(&self, rt: &'static Runtime) -> RefMut<Item> {
        let items = rt.items.borrow_mut();
        RefMut::map(items, |items| {
            items.get_mut(*self).expect("item used after internal drop")
        })
    }

    pub(crate) fn value<T: 'static>(&self, rt: &'static Runtime) -> Ref<T> {
        Ref::map(self.get(rt), |item| {
            item.value
                .as_ref()
                .expect("item holds no value")
                .downcast_ref()
                .expect("dynamic item type mismatch")
        })
    }

    pub(crate) fn value_mut<T: 'static>(&self, rt: &'static Runtime) -> RefMut<T> {
        RefMut::map(self.get_mut(rt), |item| {
            item.value
                .as_mut()
                .expect("item holds no value")
                .downcast_mut()
                .expect("dynamic item type mismatch")
        })
    }

    pub(crate) fn run(&self, rt: &'static Runtime) {
        rt.active.borrow_mut().push(*self);

        let mut action = mem::take(&mut self.get_mut(rt).action);
        (action.as_deref_mut().expect("item holds no action"))();
        self.get_mut(rt).action = action;

        rt.active.borrow_mut().pop();

        for key in mem::take(&mut rt.scopes.borrow_mut().remove(*self).unwrap_or_default()) {
            rt.remove(key);
        }
    }

    pub(crate) fn track(&self, rt: &'static Runtime) {
        if rt.not_tracking.get() {
            return;
        }

        if let Some(active) = rt.active() {
            let mut item = self.get_mut(rt);
            if !item.dependents.contains(&active) {
                item.dependents.push(active);
            }
        }
    }

    pub(crate) fn trigger(&self, rt: &'static Runtime) {
        let dependents = mem::take(&mut self.get_mut(rt).dependents);

        for key in dependents {
            key.run(rt);
        }
    }
}

pub trait Read<T: 'static> {
    fn track(&self);

    fn with<U, F: FnOnce(&T) -> U>(&self, f: F) -> U;

    fn get(&self) -> T
    where
        T: Copy,
    {
        self.with(|value| *value)
    }
}

pub trait Write<T: 'static> {
    fn trigger(&self);

    fn update<U, F: FnOnce(&mut T) -> U>(&self, f: F) -> U;

    fn replace(&self, value: T) -> T {
        self.update(|current| mem::replace(current, value))
    }

    fn set(&self, value: T) {
        self.replace(value);
    }
}

pub trait Run {
    fn run(&self);
}
