use std::collections::HashSet;

use crate::runtime::{Handle, Runtime};

pub struct Scope {
    handle: Handle,
}

impl Default for Scope {
    fn default() -> Self {
        Self {
            handle: Handle::scoped(),
        }
    }
}

impl Scope {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn run<T, F>(&self, f: F) -> T
    where
        F: FnOnce() -> T,
    {
        self.handle.with(|rt| {
            let prev = rt.current_scope.replace(self.handle);
            let value = f();
            rt.current_scope.set(prev);
            value
        })
    }

    fn try_dispose(&self) {
        self.handle
            .try_with(|rt| {
                if let Some(handles) = rt
                    .scopes
                    .try_borrow_mut()
                    .ok()
                    .and_then(|mut scopes| scopes.remove(&self.handle))
                {
                    dispose_set(rt, handles);
                }
            })
            .ok();
    }
}

impl Drop for Scope {
    fn drop(&mut self) {
        self.try_dispose();
    }
}

pub fn with_scope<T, F>(f: F) -> T
where
    F: FnOnce() -> T,
{
    Scope::new().run(f)
}

fn dispose_set(rt: &Runtime, handles: HashSet<Handle>) {
    for handle in handles {
        if let Some(scope) = rt.scopes.borrow_mut().remove(&handle) {
            dispose_set(rt, scope);
        }

        rt.signals.borrow_mut().remove(&handle);
    }
}
