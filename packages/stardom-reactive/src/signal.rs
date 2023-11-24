use std::marker::PhantomData;

use crate::{item::Item, ItemKey, Read, Runtime, Write};

pub struct Signal<T: 'static> {
    rt: &'static Runtime,
    key: ItemKey,
    _phantom: PhantomData<T>,
}

impl<T: 'static> Signal<T> {
    pub fn new(rt: &'static Runtime, parent: Option<ItemKey>, value: T) -> Self {
        let item = Item {
            value: Some(Box::new(value)),
            parent,
            ..Default::default()
        };

        Self {
            rt,
            key: rt.add(item),
            _phantom: PhantomData,
        }
    }
}

impl<T: 'static> Read<T> for Signal<T> {
    fn track(&self) {
        self.key.track(self.rt);
    }

    fn with<U, F: FnOnce(&T) -> U>(&self, f: F) -> U {
        self.track();
        f(&*self.key.value(self.rt))
    }
}

impl<T: 'static> Write<T> for Signal<T> {
    fn trigger(&self) {
        self.key.trigger(self.rt);
    }

    fn update<U, F: FnOnce(&mut T) -> U>(&self, f: F) -> U {
        let value = f(&mut *self.key.value_mut(self.rt));
        self.trigger();
        value
    }
}

impl<T> Clone for Signal<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for Signal<T> {}
