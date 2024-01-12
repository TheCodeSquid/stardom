mod item;
mod memo;
mod signal;

use std::{
    cell::{Cell, RefCell},
    thread_local,
};

use slotmap::SlotMap;

pub(crate) use item::{Item, ItemKey};
pub use memo::Memo;
pub use signal::Signal;

thread_local! {
    static GLOBAL: Cell<Option<&'static Runtime>> = const { Cell::new(None) };
}

pub(crate) struct Runtime {
    pub(crate) items: RefCell<SlotMap<ItemKey, Item>>,

    pub(crate) tracking: Cell<bool>,
    pub(crate) stack: RefCell<Vec<ItemKey>>,
}

impl Default for Runtime {
    fn default() -> Self {
        Self {
            items: RefCell::default(),
            tracking: Cell::new(true),
            stack: RefCell::default(),
        }
    }
}

impl Runtime {
    pub fn init() -> bool {
        if GLOBAL.get().is_some() {
            return true;
        }
        let leaked = Box::leak(Box::default());
        GLOBAL.set(Some(leaked));
        false
    }

    pub fn global() -> Option<&'static Self> {
        GLOBAL.get()
    }

    pub(crate) fn unwrap() -> &'static Self {
        Self::global().expect("no current reactive runtime")
    }
}

pub fn untrack<T, F>(f: F) -> T
where
    F: FnOnce() -> T,
{
    let rt = Runtime::unwrap();
    let prev = rt.tracking.replace(true);
    let value = f();
    rt.tracking.set(prev);
    value
}

pub fn effect<F>(f: F)
where
    F: FnMut() + 'static,
{
    let (rt, key) = Item::create(|item| item.action = Some(Box::new(f)));
    key.run(rt);
}

pub fn signal<T: 'static>(value: T) -> Signal<T> {
    Signal::new(value)
}

pub fn memo<T, F>(f: F) -> Memo<T>
where
    T: 'static,
    F: Fn() -> T + 'static,
{
    Memo::new(f)
}

pub trait Input<T> {
    fn track(&self);

    fn with<U, F>(&self, f: F) -> U
    where
        F: FnOnce(&T) -> U;

    fn get(&self) -> T
    where
        T: Copy,
    {
        self.with(Clone::clone)
    }
}

pub trait Output<T> {
    fn trigger(&self);

    fn update<U, F>(&self, f: F) -> U
    where
        F: FnOnce(&mut T) -> U;

    fn replace(&self, value: T) -> T {
        self.update(|current| std::mem::replace(current, value))
    }

    fn set(&self, value: T) {
        self.replace(value);
    }
}