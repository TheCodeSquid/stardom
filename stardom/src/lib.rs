#![warn(clippy::use_self)]

mod component;
pub mod constants;
mod node;

use std::thread_local;
use wasm_bindgen::JsCast;

pub use crate::{
    component::{on_mount, on_unmount},
    node::{Node, WeakNode},
};
pub use stardom_macros::{component, element};

thread_local! {
    static DOCUMENT: Option<web_sys::Document> = if cfg!(target_family = "wasm") {
        web_sys::window().and_then(|window| window.document())
    } else {
        None
    };
}

pub fn document() -> Option<web_sys::Document> {
    DOCUMENT.with(Clone::clone)
}

pub fn mount(node: Node, selector: &str) {
    let target = document()
        .expect("can only mount in browser environments")
        .query_selector(selector)
        .unwrap()
        .unwrap_or_else(|| panic!("selector '{}' matched no elements", selector));
    node.mount(&target, None);
    node.mark_main();
    std::mem::forget(node);
}

pub trait EventKey {
    type Value: JsCast;

    fn name(&self) -> &str;
}

impl EventKey for &str {
    type Value = web_sys::Event;

    fn name(&self) -> &str {
        self
    }
}

impl EventKey for String {
    type Value = web_sys::Event;

    fn name(&self) -> &str {
        self
    }
}
