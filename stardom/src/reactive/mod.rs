mod effect;
mod handle;
mod memo;
mod runtime;
mod scope;
mod signal;

use std::mem;

pub use self::{
    effect::effect,
    memo::{memo, Memo},
    scope::{with_scope, Scope},
    signal::{signal, Signal},
};
pub(crate) use runtime::{Runtime, RUNTIME};

pub trait Track {
    fn track(&self);
}

pub trait Trigger {
    fn trigger(&self);
}

pub trait Input<T: 'static> {
    fn with<U, F>(&self, f: F) -> U
    where
        F: FnOnce(&T) -> U;

    fn cloned(&self) -> T
    where
        T: Clone,
    {
        self.with(Clone::clone)
    }

    fn get(&self) -> T
    where
        T: Copy,
    {
        self.with(|v| *v)
    }
}

pub trait Output<T: 'static> {
    fn update<U, F>(&self, f: F) -> U
    where
        F: FnOnce(&mut T) -> U;

    fn replace(&self, value: T) -> T {
        self.update(|old| mem::replace(old, value))
    }

    fn set(&self, value: T) {
        self.replace(value);
    }
}
