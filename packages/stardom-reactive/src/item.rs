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
pub struct Item {
    pub value: Option<Box<dyn Any>>,
    pub action: Option<Box<dyn Fn()>>,

    pub dependents: Vec<ItemKey>,
}

impl ItemKey {
    pub(crate) fn get(self, rt: &'static Runtime) -> Ref<Item> {
        let items = rt.items.borrow();
        Ref::map(items, |items| {
            items.get(self).expect("item used after internal drop")
        })
    }

    pub(crate) fn get_mut(self, rt: &'static Runtime) -> RefMut<Item> {
        let items = rt.items.borrow_mut();
        RefMut::map(items, |items| {
            items.get_mut(self).expect("item used after internal drop")
        })
    }

    pub fn value<T: 'static>(self, rt: &'static Runtime) -> Ref<T> {
        let item = self.get(rt);
        Ref::map(item, |item| {
            item.value
                .as_ref()
                .unwrap()
                .downcast_ref()
                .expect("value type mismatch")
        })
    }

    pub fn value_mut<T: 'static>(self, rt: &'static Runtime) -> RefMut<T> {
        let item = self.get_mut(rt);
        RefMut::map(item, |item| {
            item.value
                .as_mut()
                .unwrap()
                .downcast_mut()
                .expect("value type mismatch")
        })
    }

    pub fn track(self, rt: &'static Runtime, dependent: Self) {
        let mut item = self.get_mut(rt);
        if !item.dependents.contains(&dependent) {
            item.dependents.push(dependent);
        }
    }

    pub fn trigger(self, rt: &'static Runtime) {
        let dependents = {
            let mut item = self.get_mut(rt);
            mem::take(&mut item.dependents)
        };

        if rt.active.borrow().is_empty() {
            for key in dependents {
                key.run(rt);
            }
        } else {
            let mut queue = rt.queue.borrow_mut();
            for key in dependents {
                if !queue.contains(&key) {
                    queue.push(key);
                }
            }
        }
    }

    pub fn run(self, rt: &'static Runtime) {
        let action = self.get_mut(rt).action.take().unwrap();
        rt.active.borrow_mut().push(self);

        action();

        rt.active.borrow_mut().pop();
        self.get_mut(rt).action.replace(action);

        let queue = mem::take(&mut *rt.queue.borrow_mut());
        for item in queue {
            item.run(rt);
        }

        let mut scopes = rt.scopes.borrow_mut();
        if let Some(list) = scopes.remove(self) {
            let mut items = rt.items.borrow_mut();
            for key in list {
                items.remove(key);
            }
        }
    }
}
