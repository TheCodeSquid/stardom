#![warn(clippy::use_self)]

pub mod attr;
pub mod component;
pub mod event;
pub mod node;
pub mod reactive;
pub mod util;

mod web;

pub use web::{document, is_web};

pub mod prelude {
    pub use crate::{
        component::{on_mount, on_unmount},
        node::{component, element, fragment, Node},
        reactive::{effect, memo, signal, Input as _, Output as _, Track as _, Trigger as _},
        util::{elements::*, lists::keyed},
    };
}

use node::Node;
use reactive::{Runtime, RUNTIME};

pub fn mount<F>(f: F, selector: &str)
where
    F: FnOnce() -> Node,
{
    if RUNTIME.with_borrow(|opt| opt.is_some()) {
        panic!("stardom application already mounted");
    }
    RUNTIME.with_borrow_mut(|opt| *opt = Some(Runtime::default()));

    let native_root = document()
        .expect("mounting only works in browser environments")
        .query_selector(selector)
        .unwrap()
        .unwrap_or_else(|| panic!("selector `{}` matched no elements", selector));
    let root = Node::from_element(native_root);

    root.insert(&f(), None);
    root.mark_main();

    std::mem::forget(root);
}

#[cfg(all(test, target_family = "wasm"))]
wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);
