use std::{
    any::{type_name, Any},
    cell::RefCell,
    marker::PhantomData,
    mem,
    rc::Rc,
};

use indexmap::IndexMap;

use crate::{effect::Effect, runtime::Handle, Input, Output, Track, Trigger};

pub fn signal<T: 'static>(value: T) -> Signal<T> {
    Signal::new(Handle::scoped(), value)
}

pub struct Signal<T> {
    handle: Handle,
    _phantom: PhantomData<T>,
}

impl<T: 'static> Signal<T> {
    pub(crate) fn new(handle: Handle, value: T) -> Self {
        let raw = RawSignal::new(value);
        handle.set_signal(raw);
        Self {
            handle,
            _phantom: PhantomData,
        }
    }
}

impl<T> Track for Signal<T> {
    fn track(&self) {
        self.handle.with(|rt| {
            if !rt.tracking.get() {
                return;
            }

            if let Some(effect) = &*rt.current_effect.borrow() {
                let raw = self.handle.signal();
                effect.add_signal(self.handle);
                raw.deps
                    .borrow_mut()
                    .insert(effect.handle(), effect.clone());
            }
        });
    }
}

impl<T> Trigger for Signal<T> {
    fn trigger(&self) {
        self.handle.with(|rt| {
            let deps = mem::take(&mut *self.handle.signal().deps.borrow_mut());
            if rt.batching.get() {
                rt.effect_queue.borrow_mut().extend(deps);
            } else {
                for effect in deps.values() {
                    effect.run();
                }
            }
        })
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
            .signal()
            .value
            .borrow()
            .downcast_ref()
            .unwrap_or_else(|| wrong_type::<T>()))
    }
}

impl<T: 'static> Output<T> for Signal<T> {
    fn update<U, F>(&self, f: F) -> U
    where
        F: FnOnce(&mut T) -> U,
    {
        let value = f(self
            .handle
            .signal()
            .value
            .borrow_mut()
            .downcast_mut()
            .unwrap_or_else(|| wrong_type::<T>()));
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

#[derive(Clone)]
pub(crate) struct RawSignal {
    value: Rc<RefCell<dyn Any>>,
    deps: Rc<RefCell<IndexMap<Handle, Rc<Effect>>>>,
}

impl RawSignal {
    pub fn new<T: 'static>(value: T) -> Self {
        Self {
            value: Rc::new(RefCell::new(value)),
            deps: Rc::default(),
        }
    }

    pub fn remove(&self, handle: Handle) {
        self.deps.borrow_mut().shift_remove(&handle);
    }
}

fn wrong_type<T>() -> ! {
    panic!(
        "internal signal value was not of type `{}`",
        type_name::<T>()
    )
}
