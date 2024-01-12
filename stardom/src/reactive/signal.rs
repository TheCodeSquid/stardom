use std::marker::PhantomData;

use super::{
    item::{Item, ItemKey},
    Input, Output, Runtime,
};

pub struct Signal<T: 'static> {
    rt: &'static Runtime,
    key: ItemKey,
    _phantom: PhantomData<*mut T>, // invariant
}

impl<T: 'static> Signal<T> {
    pub fn new(value: T) -> Self {
        let (rt, key) = Item::create(|item| item.value = Some(Box::new(value)));
        Self {
            rt,
            key,
            _phantom: PhantomData,
        }
    }
}

impl<T: 'static> Input<T> for Signal<T> {
    fn track(&self) {
        self.key.unwrap(self.rt).track();
    }

    fn with<U, F>(&self, f: F) -> U
    where
        F: FnOnce(&T) -> U,
    {
        let mut item = self.key.unwrap(self.rt);
        item.track();
        f(item.value.as_ref().unwrap().downcast_ref().unwrap())
    }
}

impl<T: 'static> Output<T> for Signal<T> {
    fn trigger(&self) {
        self.key.trigger(self.rt);
    }

    fn update<U, F>(&self, f: F) -> U
    where
        F: FnOnce(&mut T) -> U,
    {
        let value = {
            let mut item = self.key.unwrap(self.rt);
            f(item.value.as_mut().unwrap().downcast_mut().unwrap())
        };
        self.key.trigger(self.rt);
        value
    }
}

impl<T> Copy for Signal<T> {}
impl<T> Clone for Signal<T> {
    fn clone(&self) -> Self {
        *self
    }
}
