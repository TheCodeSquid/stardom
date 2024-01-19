use std::{cell::Cell, thread_local};

use super::{
    runtime::{Runtime, RUNTIME},
    signal::RawSignal,
};

thread_local! {
    static CYCLE: Cell<u64> = const { Cell::new(0) };
    static ID: Cell<u64> = const { Cell::new(0) };
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub(super) struct Handle {
    cycle: u64,
    id: u64,
}

impl Handle {
    pub(super) fn new(cycle: u64) -> Self {
        let id = ID.replace(ID.get() + 1);
        Self { cycle, id }
    }

    pub(super) fn create() -> Self {
        RUNTIME.with_borrow(|opt| {
            let Some(rt) = opt else {
                panic!("no active runtime");
            };
            Self::new(rt.handle_cycle)
        })
    }

    pub(super) fn next_cycle() -> u64 {
        ID.set(0);
        CYCLE.replace(CYCLE.get() + 1)
    }

    pub(super) fn runtime<T, F>(&self, f: F) -> T
    where
        F: FnOnce(&Runtime) -> T,
    {
        RUNTIME.with_borrow(|opt| {
            if let Some(rt) = opt {
                if rt.handle_cycle == self.cycle {
                    return f(rt);
                }
            }
            panic!("reactive item used outside of its runtime");
        })
    }

    pub(super) fn bind_scope(&self) {
        self.runtime(|rt| {
            let scope_handle = rt.current_scope.get();
            let mut scopes = rt.scopes.borrow_mut();
            scopes.entry(scope_handle).or_default().insert(*self);
        });
    }

    pub(super) fn init_signal(&self, raw: RawSignal) {
        self.runtime(|rt| {
            rt.signals.borrow_mut().insert(*self, raw);
        });
    }

    pub(super) fn raw_signal(&self) -> RawSignal {
        self.runtime(|rt| {
            rt.signals
                .borrow()
                .get(self)
                .expect("reactive item used outside of its scope")
                .clone()
        })
    }

    pub(super) fn discard(&self) {
        RUNTIME
            .try_with(|cell| {
                if let Some(rt) = &*cell.borrow() {
                    if let Some(scope) = rt.scopes.borrow_mut().remove(self) {
                        for handle in scope {
                            handle.discard();
                        }
                    }

                    rt.signals.borrow_mut().remove(self);
                }
            })
            .ok();
    }
}
