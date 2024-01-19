use std::{any::Any, cell::RefCell, marker::PhantomData, mem, rc::Rc};

use indexmap::IndexMap;

use super::{effect::Effect, handle::Handle, Input, Output, Track, Trigger};

pub fn signal<T: 'static>(value: T) -> Signal<T> {
    Signal::new(value)
}

pub struct Signal<T: 'static> {
    pub(super) handle: Handle,
    pub(super) _phantom: PhantomData<T>,
}

#[derive(Clone)]
pub(super) struct RawSignal {
    pub(super) value: Rc<RefCell<dyn Any>>,
    pub(super) dependents: Rc<RefCell<IndexMap<Handle, Rc<Effect>>>>,
}

impl<T: 'static> Signal<T> {
    fn new(value: T) -> Self {
        let handle = Handle::create();
        handle.bind_scope();

        let raw = RawSignal {
            value: Rc::new(RefCell::new(value)),
            dependents: Rc::default(),
        };
        handle.init_signal(raw);

        Self {
            handle,
            _phantom: PhantomData,
        }
    }
}

impl<T> Track for Signal<T> {
    fn track(&self) {
        self.handle.runtime(|rt| {
            if let Some(effect) = &*rt.current_effect.borrow() {
                let raw = self.handle.raw_signal();
                let mut deps = raw.dependents.borrow_mut();
                deps.insert(effect.handle, effect.clone());
            }
        });
    }
}

impl<T> Trigger for Signal<T> {
    fn trigger(&self) {
        let raw = self.handle.raw_signal();
        let deps = mem::take(&mut *raw.dependents.borrow_mut());
        for effect in deps.values() {
            effect.run();
        }
    }
}

impl<T: 'static> Input<T> for Signal<T> {
    fn with<U, F>(&self, f: F) -> U
    where
        F: FnOnce(&T) -> U,
    {
        self.track();
        f(self
            .handle
            .raw_signal()
            .value
            .borrow()
            .downcast_ref()
            .expect("signal type mismatch"))
    }
}

impl<T: 'static> Output<T> for Signal<T> {
    fn update<U, F>(&self, f: F) -> U
    where
        F: FnOnce(&mut T) -> U,
    {
        let value = f(self
            .handle
            .raw_signal()
            .value
            .borrow_mut()
            .downcast_mut()
            .expect("signal type mismatch"));
        self.trigger();
        value
    }
}

impl<T> Copy for Signal<T> {}
impl<T> Clone for Signal<T> {
    fn clone(&self) -> Self {
        *self
    }
}
