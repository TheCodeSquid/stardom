use std::mem;

use crate::{
    env::{self, Env},
    node::Node,
};

pub fn mount<N, F>(root: N, f: F)
where
    N: Into<web_sys::Node>,
    F: FnOnce() -> Node,
{
    env::replace(Env::Browser);
    stardom_reactive::run(|_| {
        let node = Node::fragment();
        node.manual_bind(root);
        node.insert(&f(), None);
        node.set_main_tree(true);
        mem::forget(node);
    });
}
