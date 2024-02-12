use std::{
    cell::{Cell, RefCell},
    collections::{HashMap, HashSet},
    mem,
    rc::Rc,
    thread::AccessError,
    thread_local,
};

use indexmap::IndexMap;

use crate::{effect::Effect, signal::RawSignal};

thread_local! {
    static CYCLE: Cell<u64> = const { Cell::new(0) };
    static ID: Cell<u64> = const { Cell::new(0) };
    static STACK: RefCell<Vec<Runtime>> = RefCell::default();
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub(crate) struct Handle {
    cycle: u64,
    id: u64,
}

impl Handle {
    pub fn new(cycle: u64) -> Self {
        let id = ID.replace(ID.get() + 1);
        Self { cycle, id }
    }

    pub fn next() -> Self {
        Runtime::with(|rt| Self::new(rt.cycle))
    }

    pub fn scoped() -> Self {
        let handle = Self::next();
        handle.bind_scope();
        handle
    }

    pub fn try_with<T, F>(&self, f: F) -> Result<T, AccessError>
    where
        F: FnOnce(&Runtime) -> T,
    {
        Runtime::try_with(|rt| {
            if rt.cycle != self.cycle {
                panic!("reactive item used outside of its runtime");
            }
            f(rt)
        })
    }

    pub fn with<T, F>(&self, f: F) -> T
    where
        F: FnOnce(&Runtime) -> T,
    {
        Runtime::with(|rt| {
            if rt.cycle != self.cycle {
                panic!("reactive item used outside of its runtime");
            }
            f(rt)
        })
    }

    pub fn bind_scope(&self) {
        self.with(|rt| {
            let scope = rt.current_scope.get();
            rt.scopes
                .borrow_mut()
                .entry(scope)
                .or_default()
                .insert(*self);
        });
    }

    pub fn set_signal(&self, raw: RawSignal) {
        self.with(|rt| {
            rt.signals.borrow_mut().insert(*self, raw);
        });
    }

    pub fn signal(&self) -> RawSignal {
        self.with(|rt| {
            rt.signals
                .borrow()
                .get(self)
                .cloned()
                .expect("no signal assigned to handle")
        })
    }
}

pub(crate) struct Runtime {
    pub scopes: RefCell<HashMap<Handle, HashSet<Handle>>>,
    pub signals: RefCell<HashMap<Handle, RawSignal>>,

    pub cycle: u64,
    pub tracking: Cell<bool>,
    pub batching: Cell<bool>,

    pub current_scope: Cell<Handle>,
    pub current_effect: RefCell<Option<Rc<Effect>>>,
    pub effect_queue: RefCell<IndexMap<Handle, Rc<Effect>>>,
}

impl Runtime {
    fn new() -> Self {
        let cycle = CYCLE.replace(CYCLE.get() + 1);
        ID.set(0);
        Self {
            scopes: RefCell::default(),
            signals: RefCell::default(),
            cycle,
            tracking: Cell::new(true),
            batching: Cell::new(false),
            current_scope: Cell::new(Handle::new(cycle)),
            current_effect: RefCell::default(),
            effect_queue: RefCell::default(),
        }
    }

    pub fn try_with<T, F>(f: F) -> Result<T, AccessError>
    where
        for<'a> F: FnOnce(&'a Self) -> T,
    {
        STACK.try_with(|cell| f(cell.borrow().last().expect("not within reactive runtime")))
    }

    pub fn with<T, F>(f: F) -> T
    where
        for<'a> F: FnOnce(&'a Self) -> T,
    {
        STACK.with_borrow(|stack| f(stack.last().expect("not within reactive runtime")))
    }

    fn trigger_queued(&self) {
        let queued = mem::take(&mut *self.effect_queue.borrow_mut());
        for effect in queued.values() {
            effect.run();
        }
    }
}

pub fn run<T, F>(f: F) -> T
where
    F: FnOnce(fn()) -> T,
{
    STACK.with_borrow_mut(|stack| stack.push(Runtime::new()));
    f(|| {
        STACK.with_borrow_mut(Vec::pop);
    })
}

pub fn untrack<T, F>(f: F) -> T
where
    F: FnOnce() -> T,
{
    Runtime::with(|rt| {
        let prev = rt.tracking.replace(false);
        let value = f();
        rt.tracking.set(prev);
        value
    })
}

pub fn batch<T, F>(f: F) -> T
where
    F: FnOnce() -> T,
{
    Runtime::with(|rt| {
        let prev = rt.batching.replace(true);
        let value = f();
        rt.batching.set(prev);
        if !prev {
            rt.trigger_queued();
        }
        value
    })
}
