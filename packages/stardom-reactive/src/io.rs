use std::mem;

use crate::{memo::Memo, runtime::Runtime};

pub trait Track<T: 'static> {
    fn runtime(&self) -> &'static Runtime;

    fn track(&self);

    fn with<U, F>(&self, f: F) -> U
    where
        F: FnOnce(&T) -> U;

    fn get(&self) -> T
    where
        T: Copy,
    {
        self.with(|v| *v)
    }

    fn map<U, F>(&self, f: F) -> Memo<U>
    where
        Self: Clone + 'static,
        U: 'static,
        F: Fn(&T) -> U + 'static,
    {
        let s = self.clone();
        self.runtime().memo(move || s.with(|v| f(v)))
    }
}

pub trait Trigger<T: 'static> {
    fn trigger(&self);

    fn update<U, F>(&self, f: F) -> U
    where
        F: FnOnce(&mut T) -> U;

    fn replace(&self, value: T) -> T {
        self.update(|v| mem::replace(v, value))
    }

    fn set(&self, value: T) {
        self.replace(value);
    }
}
