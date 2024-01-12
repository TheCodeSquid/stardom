use std::marker::PhantomData;

use super::{
    item::{Item, ItemKey},
    Input, Runtime,
};

pub struct Memo<T: 'static> {
    rt: &'static Runtime,
    key: ItemKey,
    _phantom: PhantomData<*mut T>, // invariant
}

impl<T: 'static> Memo<T> {
    pub fn new<F>(f: F) -> Self
    where
        F: Fn() -> T + 'static,
    {
        let write = move |rt: &'static Runtime, key: ItemKey| {
            key.unwrap(rt).value = Some(Box::new(f()));
            key.trigger(rt);
        };

        let (rt, key) = Item::create(|item| {
            let Item { rt, key, .. } = *item;
            item.action = Some(Box::new(move || {
                write(rt, key);
            }));
        });
        key.run(rt);

        Self {
            rt,
            key,
            _phantom: PhantomData,
        }
    }
}

impl<T: 'static> Input<T> for Memo<T> {
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

impl<T> Copy for Memo<T> {}
impl<T> Clone for Memo<T> {
    fn clone(&self) -> Self {
        *self
    }
}
