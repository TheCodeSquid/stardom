use std::{
    cell::{Ref, RefCell},
    marker::PhantomData,
    rc::{Rc, Weak},
};

use super::{item::Item, Input};

pub struct Memo<T: 'static> {
    item: Rc<Item>,
    _phantom: PhantomData<*mut T>, // invariant
}

impl<T: 'static> Memo<T> {
    pub fn new<F>(f: F) -> Self
    where
        F: Fn() -> T + 'static,
    {
        let item = Rc::new_cyclic(move |weak: &Weak<Item>| {
            let weak = weak.clone();

            let action = move || {
                let value = f();
                let item = weak.upgrade().unwrap();
                *item.value.as_ref().unwrap().borrow_mut() = Box::new(value);
                item.trigger();
            };

            Item {
                value: Some(RefCell::new(Box::new(()))),
                action: Some(Box::new(action)),
                ..Item::new()
            }
        });
        item.run();

        Self {
            item,
            _phantom: PhantomData,
        }
    }

    fn value(&self) -> Ref<T> {
        let value = self.item.value.as_ref().unwrap();
        Ref::map(value.borrow(), |v| v.downcast_ref().unwrap())
    }
}

impl<T: 'static> Input<T> for Memo<T> {
    fn track(&self) {
        self.item.track();
    }

    fn with<U, F>(&self, f: F) -> U
    where
        F: FnOnce(&T) -> U,
    {
        self.track();
        f(&*self.value())
    }
}

impl<T: 'static> Clone for Memo<T> {
    fn clone(&self) -> Self {
        Self {
            item: self.item.clone(),
            _phantom: self._phantom,
        }
    }
}
