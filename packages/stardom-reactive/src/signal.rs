use std::marker::PhantomData;

use crate::{
    item::{Item, ItemKey},
    HasRuntime, Runnable, Runtime, Track, Trigger,
};

pub struct Signal<T: 'static> {
    rt: &'static Runtime,
    item: ItemKey,
    _phantom: PhantomData<T>,
}

impl<T: 'static> Signal<T> {
    pub(crate) fn new(rt: &'static Runtime, value: T) -> Self {
        let item = Item {
            value: Some(Box::new(value)),
            ..Default::default()
        };
        let key = rt.register(item);

        Self {
            rt,
            item: key,
            _phantom: PhantomData,
        }
    }
}

impl<T: 'static> Clone for Signal<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T: 'static> Copy for Signal<T> {}

impl<T: 'static> HasRuntime for Signal<T> {
    fn runtime(&self) -> &'static Runtime {
        self.rt
    }
}

impl<T: 'static> Track<T> for Signal<T> {
    fn track<R: Runnable>(&self, runnable: &R) {
        self.item.track(self.rt, runnable.item_key());
    }

    fn with<U, F>(&self, f: F) -> U
    where
        F: FnOnce(&T) -> U,
    {
        let value = f(&*self.item.value(self.rt));
        self.track_active();
        value
    }
}

impl<T: 'static> Trigger<T> for Signal<T> {
    fn trigger(&self) {
        self.item.trigger(self.rt);
    }

    fn update<U, F>(&self, f: F) -> U
    where
        F: FnOnce(&mut T) -> U,
    {
        let value = f(&mut *self.item.value_mut(self.rt));
        self.trigger();
        value
    }
}
