use crate::{effect::Effect, runtime::Handle, signal::Signal, Input, Output, Track};

pub fn memo<T, F>(f: F) -> Memo<T>
where
    T: 'static,
    F: FnMut() -> T + 'static,
{
    Memo::new(f)
}

pub struct Memo<T: 'static> {
    signal: Signal<Option<T>>,
}

impl<T: 'static> Memo<T> {
    pub(crate) fn new<F>(mut f: F) -> Self
    where
        F: FnMut() -> T + 'static,
    {
        let handle = Handle::scoped();
        let signal = Signal::new(handle, None);

        Effect::new(handle, move || {
            signal.set(Some(f()));
        })
        .run();

        Self { signal }
    }
}

impl<T> Track for Memo<T> {
    fn track(&self) {
        self.signal.track();
    }
}

impl<T> Input<T> for Memo<T> {
    fn with<U, F>(&self, f: F) -> U
    where
        F: FnOnce(&T) -> U,
    {
        self.signal
            .with(|option| f(option.as_ref().expect("uninitialized memo")))
    }
}

impl<T> Copy for Memo<T> {}
impl<T> Clone for Memo<T> {
    fn clone(&self) -> Self {
        *self
    }
}
