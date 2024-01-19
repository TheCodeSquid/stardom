use std::{cell::RefCell, collections::HashSet, rc::Rc};

use super::{handle::Handle, scope::with_scope};

pub fn effect<F>(f: F)
where
    F: Fn() + 'static,
{
    Effect::create(f);
}

pub(super) struct Effect {
    pub(super) handle: Handle,
    pub(super) f: Box<dyn Fn()>,
    pub(super) dependencies: RefCell<HashSet<Handle>>,
}

impl Effect {
    fn create<F>(f: F)
    where
        F: Fn() + 'static,
    {
        let handle = Handle::create();
        handle.bind_scope();

        let effect = Rc::new(Self {
            handle,
            f: Box::new(f),
            dependencies: RefCell::default(),
        });
        effect.run();
    }

    pub(super) fn run(self: &Rc<Self>) {
        self.clean_dependencies();

        self.handle.runtime(|rt| {
            let prev = rt.current_effect.replace(Some(self.clone()));

            with_scope(&self.f);

            *rt.current_effect.borrow_mut() = prev;
        });
    }

    fn clean_dependencies(&self) {
        for handle in &*self.dependencies.borrow() {
            let raw = handle.raw_signal();
            raw.dependents.borrow_mut().remove(&self.handle);
        }
    }
}
