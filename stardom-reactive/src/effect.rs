use std::{cell::RefCell, collections::HashSet, mem, rc::Rc};

use crate::{
    runtime::{batch, Handle, Runtime},
    scope::with_scope,
    Track,
};

pub fn effect<F>(f: F)
where
    F: FnMut() + 'static,
{
    Effect::new(Handle::scoped(), f).run();
}

pub fn lazy_effect<F>(f: F) -> LazyEffect
where
    F: FnMut() + 'static,
{
    LazyEffect(Effect::new(Handle::scoped(), f))
}

#[derive(Clone)]
pub struct LazyEffect(Rc<Effect>);

impl LazyEffect {
    pub fn add<T: Track>(&self, tracker: &T) {
        self.0.handle.with(|rt| {
            let prev = rt.current_effect.borrow_mut().replace(self.0.clone());
            tracker.track();
            *rt.current_effect.borrow_mut() = prev;
        });
    }

    pub fn run(&self) {
        self.0.run();
    }
}

pub(crate) struct Effect {
    handle: Handle,
    f: Box<RefCell<dyn FnMut()>>,
    deps: RefCell<HashSet<Handle>>,
}

impl Effect {
    pub fn new<F>(handle: Handle, f: F) -> Rc<Self>
    where
        F: FnMut() + 'static,
    {
        Rc::new(Self {
            handle,
            f: Box::new(RefCell::new(f)),
            deps: RefCell::default(),
        })
    }

    pub fn handle(&self) -> Handle {
        self.handle
    }

    pub fn add_signal(&self, handle: Handle) {
        self.deps.borrow_mut().insert(handle);
    }

    pub fn run(self: &Rc<Self>) {
        self.handle.with(|rt| {
            self.clear_deps(rt);
            let prev = rt.current_effect.replace(Some(self.clone()));
            with_scope(|| batch(&mut *self.f.borrow_mut()));
            rt.current_effect.replace(prev);
        });
    }

    fn clear_deps(&self, rt: &Runtime) {
        let deps = mem::take(&mut *self.deps.borrow_mut());
        for handle in deps {
            if let Some(raw) = rt.signals.borrow().get(&handle) {
                raw.remove(self.handle);
            }
        }
    }
}
