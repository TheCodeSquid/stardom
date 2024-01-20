use std::{cell::RefCell, hash::Hash, mem};

use indexmap::IndexMap;

use crate::{
    node::Node,
    reactive::{effect, Input},
};

pub fn keyed<T, U, I, Key, K, F>(input: I, key: K, f: F) -> Node
where
    U: 'static,
    for<'a> &'a U: IntoIterator<Item = &'a T>,
    I: Input<U> + 'static,
    Key: Hash + Eq + 'static,
    K: Fn(&T) -> Key + 'static,
    F: Fn(&T) -> Node + 'static,
{
    let base = Node::fragment();
    let children = RefCell::new(IndexMap::<Key, Node>::new());

    effect({
        let base = base.clone();
        move || {
            input.with(|iter| {
                let mut children = children.borrow_mut();
                let mut prev = mem::take(&mut *children);

                for value in iter {
                    let key = key(value);

                    let old = prev.remove(&key);
                    if let Some(old) = &old {
                        base.remove(old);
                    }

                    let node = old.unwrap_or_else(|| f(value));
                    base.insert(&node, None);
                    children.insert(key, node);
                }

                for node in prev.values() {
                    base.remove(node);
                }
            })
        }
    });

    base
}
