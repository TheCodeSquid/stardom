use std::{
    cell::{Cell, RefCell},
    thread_local,
};

use slotmap::{SlotMap, SparseSecondaryMap};

use crate::{
    item::{Item, ItemKey, Run},
    Effect, Signal,
};

thread_local! {
    static RUNTIME: Cell<Option<&'static Runtime>> = const { Cell::new(None) };
}

#[derive(Default)]
pub struct Runtime {
    pub(crate) items: RefCell<SlotMap<ItemKey, Item>>,
    pub(crate) scopes: RefCell<SparseSecondaryMap<ItemKey, Vec<ItemKey>>>,

    pub(crate) not_tracking: Cell<bool>,
    pub(crate) active: RefCell<Vec<ItemKey>>,
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

    pub fn untrack<T, F: FnOnce() -> T>(&self, f: F) -> T {
        let prev = self.not_tracking.replace(true);
        let value = f();
        self.not_tracking.set(prev);
        value
    }

    pub fn active(&self) -> Option<ItemKey> {
        self.active.borrow().last().copied()
    }

    pub(crate) fn add(&self, item: Item) -> ItemKey {
        let parent = item.parent;
        let key = self.items.borrow_mut().insert(item);
        if let Some(parent) = parent {
            self.scopes
                .borrow_mut()
                .entry(parent)
                .expect("item used after internal drop")
                .and_modify(|scope| scope.push(key))
                .or_insert_with(|| vec![key]);
        }
        key
    }

    pub(crate) fn remove(&self, key: ItemKey) {
        if let Some(scope) = self.scopes.borrow_mut().remove(key) {
            for key in scope {
                self.remove(key);
            }
        }

        self.items.borrow_mut().remove(key);
    }
}

pub fn init() {
    Runtime::new().init();
}

pub fn untrack<T, F: FnOnce() -> T>(f: F) -> T {
    Runtime::unwrap_global().untrack(f)
}

pub fn signal_in<T: 'static>(parent: Option<ItemKey>, value: T) -> Signal<T> {
    Signal::new(Runtime::unwrap_global(), parent, value)
}

pub fn signal<T: 'static>(value: T) -> Signal<T> {
    let rt = Runtime::unwrap_global();
    Signal::new(rt, rt.active(), value)
}

pub fn effect_in<F: FnMut() + 'static>(parent: Option<ItemKey>, f: F) -> Effect {
    let effect = Effect::new(Runtime::unwrap_global(), parent, f);
    effect.run();
    effect
}

pub fn effect<F: FnMut() + 'static>(f: F) -> Effect {
    let rt = Runtime::unwrap_global();
    let effect = Effect::new(rt, rt.active(), f);
    effect.run();
    effect
}
