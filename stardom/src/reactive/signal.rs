use std::{
    cell::{Ref, RefCell, RefMut},
    marker::PhantomData,
    rc::Rc,
};

use super::{item::Item, Input, Output};

pub struct Signal<T: 'static> {
    item: Rc<Item>,
    _phantom: PhantomData<*mut T>, // invariant
}

impl<T: 'static> Signal<T> {
    pub fn new(value: T) -> Self {
        let item = Rc::new(Item {
            value: Some(RefCell::new(Box::new(value))),
            ..Item::new()
        });
        Self {
            item,
            _phantom: PhantomData,
        }
    }

    fn value(&self) -> Ref<T> {
        let value = self.item.value.as_ref().unwrap();
        Ref::map(value.borrow(), |v| v.downcast_ref().unwrap())
    }

    fn value_mut(&self) -> RefMut<T> {
        let value = self.item.value.as_ref().unwrap();
        RefMut::map(value.borrow_mut(), |v| v.downcast_mut().unwrap())
    }
}

impl<T: 'static> Input<T> for Signal<T> {
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

impl<T: 'static> Output<T> for Signal<T> {
    fn trigger(&self) {
        self.item.trigger();
    }

    fn update<U, F>(&self, f: F) -> U
    where
        F: FnOnce(&mut T) -> U,
    {
        let value = f(&mut *self.value_mut());
        self.item.trigger();
        value
    }
}

impl<T> Clone for Signal<T> {
    fn clone(&self) -> Self {
        Self {
            item: self.item.clone(),
            _phantom: self._phantom,
        }
    }
}
