use std::{
    cell::{Cell, RefCell},
    thread_local,
};

use slotmap::{SlotMap, SparseSecondaryMap};

use crate::{
    effect::Effect,
    item::{Item, ItemKey},
    memo::Memo,
    signal::Signal,
};

thread_local! {
    static RUNTIME: Cell<Option<&'static Runtime>> = const { Cell::new(None) };
}

pub struct Runtime {
    pub(crate) items: RefCell<SlotMap<ItemKey, Item>>,
    pub(crate) scopes: RefCell<SparseSecondaryMap<ItemKey, Vec<ItemKey>>>,

    pub(crate) tracking: Cell<bool>,
    pub(crate) active: RefCell<Vec<ItemKey>>,
    pub(crate) queue: RefCell<Vec<ItemKey>>,
}

impl Default for Runtime {
    fn default() -> Self {
        Self {
            items: RefCell::default(),
            scopes: RefCell::default(),
            tracking: Cell::new(true),
            active: RefCell::default(),
            queue: RefCell::default(),
        }
    }
}

impl Runtime {
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the global `Runtime`
    ///
    /// If there was another `Runtime` previously registered, it is returned.
    pub fn init(self) -> Option<&'static Self> {
        let leaked = Box::leak(Box::new(self));
        RUNTIME.replace(Some(leaked))
    }

    /// Removes and returns the active `Runtime`
    pub fn deinit() -> Option<&'static Self> {
        RUNTIME.take()
    }

    pub fn global() -> Option<&'static Self> {
        RUNTIME.get()
    }

    pub fn unwrap_global() -> &'static Self {
        Self::global().expect("no active runtime")
    }

    pub(crate) fn register(&self, item: Item) -> ItemKey {
        let key = self.items.borrow_mut().insert(item);
        self.add_to_active(key);
        key
    }

    pub(crate) fn register_with_key<F>(&self, f: F) -> ItemKey
    where
        F: FnOnce(ItemKey) -> Item,
    {
        let key = self.items.borrow_mut().insert_with_key(f);
        self.add_to_active(key);
        key
    }

    pub(crate) fn add_to_active(&self, key: ItemKey) {
        if let Some(active) = self.active.borrow().last() {
            let mut scopes = self.scopes.borrow_mut();
            scopes
                .entry(*active)
                .unwrap()
                .and_modify(|v| v.push(key))
                .or_insert(vec![key]);
        }
    }

    pub fn effect<F: Fn() + 'static>(&'static self, f: F) -> Effect {
        let effect = self.lazy_effect(f);
        effect.run();
        effect
    }

    pub fn lazy_effect<F: Fn() + 'static>(&'static self, f: F) -> Effect {
        Effect::new(self, f)
    }

    pub fn signal<T: 'static>(&'static self, value: T) -> Signal<T> {
        Signal::new(self, value)
    }

    pub fn memo<T, F>(&'static self, f: F) -> Memo<T>
    where
        T: 'static,
        F: Fn() -> T + 'static,
    {
        Memo::new(self, f)
    }
}

pub fn untrack<T, F>(f: F) -> T
where
    F: FnOnce() -> T,
{
    if let Some(rt) = Runtime::global() {
        let old = rt.tracking.replace(false);
        let value = f();
        rt.tracking.set(old);
        value
    } else {
        f()
    }
}

pub fn effect<F: Fn() + 'static>(f: F) -> Effect {
    Runtime::unwrap_global().effect(f)
}

pub fn lazy_effect<F: Fn() + 'static>(f: F) -> Effect {
    Runtime::unwrap_global().lazy_effect(f)
}

pub fn signal<T: 'static>(value: T) -> Signal<T> {
    Runtime::unwrap_global().signal(value)
}

pub fn memo<T, F>(f: F) -> Memo<T>
where
    T: 'static,
    F: Fn() -> T + 'static,
{
    Runtime::unwrap_global().memo(f)
}
