use std::{cell::Cell, thread_local};

thread_local! {
    static ENV: Cell<Env> = const { Cell::new(Env::Render) };
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum Env {
    Render,
    Browser,
    Hydrate,
}

pub(crate) fn is_browser() -> bool {
    matches!(ENV.get(), Env::Browser)
}

pub(crate) fn is_hydrating() -> bool {
    matches!(ENV.get(), Env::Hydrate)
}

pub(crate) fn replace(env: Env) -> Env {
    let browser = cfg!(target_family = "wasm") && web_sys::window().is_some();

    if !browser && matches!(env, Env::Browser | Env::Hydrate) {
        panic!("client rendering unsupported in this environment");
    }

    ENV.replace(env)
}

pub(crate) fn with<T, F>(env: Env, f: F) -> T
where
    F: FnOnce() -> T,
{
    let prev = replace(env);
    let value = f();
    replace(prev);
    value
}
