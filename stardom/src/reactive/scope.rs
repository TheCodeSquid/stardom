use super::handle::Handle;

pub fn with_scope<T, F>(f: F) -> T
where
    F: FnOnce() -> T,
{
    Scope::new().with(f)
}

pub struct Scope(Handle);

impl Scope {
    fn new() -> Self {
        Self(Handle::create())
    }

    pub fn with<T, F>(&self, f: F) -> T
    where
        F: FnOnce() -> T,
    {
        self.0.runtime(|rt| {
            let prev = rt.current_scope.replace(self.0);
            let value = f();
            rt.current_scope.set(prev);
            value
        })
    }
}

impl Drop for Scope {
    fn drop(&mut self) {
        self.0.discard();
    }
}
