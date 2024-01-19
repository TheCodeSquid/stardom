use std::{cell::RefCell, marker::PhantomData, rc::Rc};

use super::{
    effect::Effect,
    handle::Handle,
    signal::{RawSignal, Signal},
    Input, Output, Track,
};

pub fn memo<T, F>(f: F) -> Memo<T>
where
    T: 'static,
    F: Fn() -> T + 'static,
{
    Memo::new(f)
}

pub struct Memo<T: 'static>(Signal<Option<T>>);

impl<T: 'static> Memo<T> {
    fn new<F>(f: F) -> Self
    where
        F: Fn() -> T + 'static,
    {
        let handle = Handle::create();
        handle.bind_scope();

        let raw = RawSignal {
            value: Rc::new(RefCell::new(None::<T>)),
            dependents: Rc::default(),
        };
        handle.init_signal(raw);
        let signal = Signal {
            handle,
            _phantom: PhantomData,
        };

        let effect = Rc::new(Effect {
            handle,
            f: Box::new(move || {
                signal.set(Some(f()));
            }),
            dependencies: RefCell::default(),
        });
        effect.run();

        Self(signal)
    }
}

impl<T> Track for Memo<T> {
    fn track(&self) {
        self.0.track();
    }
}

impl<T: 'static> Input<T> for Memo<T> {
    fn with<U, F>(&self, f: F) -> U
    where
        F: FnOnce(&T) -> U,
    {
        self.0
            .with(|opt| f(opt.as_ref().expect("uninitialized memo")))
    }
}
