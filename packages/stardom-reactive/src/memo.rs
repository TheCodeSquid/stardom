use std::marker::PhantomData;

use crate::{
    item::{Item, ItemKey},
    HasRuntime, Runnable, Runtime, Track,
};

pub struct Memo<T: 'static> {
    rt: &'static Runtime,
    item: ItemKey,
    _phantom: PhantomData<T>,
}

impl<T: 'static> Memo<T> {
    pub(crate) fn new<F>(rt: &'static Runtime, f: F) -> Self
    where
        F: Fn() -> T + 'static,
    {
        let key = rt.register_with_key(|key| {
            let action = Box::new(move || {
                let value = f();
                key.get_mut(rt).value.replace(Box::new(value));
                key.trigger(rt);
            });

            Item {
                action: Some(action),
                ..Default::default()
            }
        });

        key.run(rt);

        Self {
            rt,
            item: key,
            _phantom: PhantomData,
        }
    }
}

impl<T: 'static> Clone for Memo<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T: 'static> Copy for Memo<T> {}

impl<T: 'static> HasRuntime for Memo<T> {
    fn runtime(&self) -> &'static Runtime {
        self.rt
    }
}

impl<T: 'static> Track<T> for Memo<T> {
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
