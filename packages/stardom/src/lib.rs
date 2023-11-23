pub use stardom_nodes as nodes;
pub use stardom_reactive as reactive;

pub use stardom_render as render;
pub use stardom_web as web;

pub use render::{render, render_string};

pub fn mount<F: FnOnce() -> web::DomNode>(f: F, selector: &str) {
    reactive::Runtime::new().init();
    let root = web::document()
        .query_selector(selector)
        .unwrap()
        .expect("selector did not match");

    let node = f();
    node.mount_to_native(&root, None);
    std::mem::forget(node);
}

pub mod prelude {
    #[doc(hidden)]
    pub use stardom_macros;
    #[doc(hidden)]
    pub use stardom_nodes;
    #[doc(hidden)]
    pub use stardom_reactive;

    pub use crate::{
        nodes::*,
        reactive::{
            effect, lazy_effect, memo, signal, untrack, Effect, Memo, Signal, Track, Trigger,
        },
    };
}
