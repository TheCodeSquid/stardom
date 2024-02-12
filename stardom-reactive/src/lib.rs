#![warn(clippy::use_self)]

mod effect;
mod memo;
mod runtime;
mod scope;
mod signal;

use std::mem;

pub use self::{effect::*, memo::*, runtime::*, scope::*, signal::*};

pub trait Track {
    fn track(&self);
}

pub trait Trigger {
    fn trigger(&self);
}

pub trait Input<T> {
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

pub trait Output<T> {
    fn update<U, F>(&self, f: F) -> U
    where
        F: FnOnce(&mut T) -> U;

    fn replace(&self, value: T) -> T {
        self.update(|current| mem::replace(current, value))
    }

    fn set(&self, value: T) {
        self.replace(value);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::{cell::Cell, rc::Rc};

    #[test]
    fn basic_reactivity() {
        run(|_| {
            let count = signal(0u8);
            let calls = Rc::new(Cell::new(0));
            let double = memo({
                let calls = calls.clone();
                move || {
                    calls.set(calls.get() + 1);
                    count.get() * 2
                }
            });

            assert_eq!(double.get(), 0);

            count.set(1);
            assert_eq!(double.get(), 2);

            count.set(2);
            assert_eq!(double.get(), 4);

            batch(|| {
                count.set(3);
                assert_eq!(double.get(), 4);
                count.set(4);
                assert_eq!(double.get(), 4);
            });
            assert_eq!(double.get(), 8);
            assert_eq!(calls.get(), 4);
        });
    }
}
