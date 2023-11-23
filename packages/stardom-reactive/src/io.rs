use std::mem;

use crate::{memo::Memo, runtime::Runtime, ItemKey};

pub trait HasRuntime {
    fn runtime(&self) -> &'static Runtime;
}

pub trait Runnable {
    fn run(&self);

    fn item_key(&self) -> ItemKey;

    fn depend<T: 'static>(&self, tracker: &impl Track<T>)
    where
        Self: Sized,
    {
        tracker.track(self);
    }
}

pub trait Track<T: 'static> {
    fn track<R: Runnable>(&self, runnable: &R);

    fn track_active(&self)
    where
        Self: HasRuntime,
    {
        let rt = self.runtime();
        if !rt.tracking.get() {
            return;
        }

        if let Some(active) = rt.active() {
            self.track(&ItemKeyWrapper(rt, active));
        }
    }

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
        Self: HasRuntime + Clone + 'static,
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

struct ItemKeyWrapper(&'static Runtime, ItemKey);

impl Runnable for ItemKeyWrapper {
    fn run(&self) {
        self.1.run(self.0);
    }

    fn item_key(&self) -> ItemKey {
        self.1
    }
}
