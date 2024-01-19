use std::{cell::RefCell, thread_local};

use crate::node::{Node, NodeKind};

thread_local! {
    static ACTIVE: RefCell<Option<Component>> = const { RefCell::new(None) };
}

pub(crate) struct Component {
    pub(crate) on_mount: Option<Box<dyn Fn()>>,
    pub(crate) on_unmount: Option<Box<dyn Fn()>>,
}

pub(crate) fn create_component<F>(f: F) -> Node
where
    F: FnOnce() -> Node,
{
    let prev = ACTIVE.replace(Some(Component {
        on_mount: None,
        on_unmount: None,
    }));

    let content = f();

    let component = ACTIVE.replace(prev).unwrap();

    let kind = NodeKind::Component(component);
    let node = Node::new(kind);
    node.insert(&content, None);
    node
}

fn with_active<F>(f: F)
where
    F: FnOnce(&mut Component),
{
    ACTIVE.with_borrow_mut(|opt| {
        f(opt
            .as_mut()
            .expect("called outside of component initialization"))
    })
}

pub fn on_mount<F>(f: F)
where
    F: Fn() + 'static,
{
    with_active(|component| {
        if let Some(existing) = component.on_mount.take() {
            component.on_mount = Some(Box::new(move || {
                existing();
                f();
            }));
        } else {
            component.on_mount = Some(Box::new(f));
        }
    });
}

pub fn on_unmount<F>(f: F)
where
    F: Fn() + 'static,
{
    with_active(|component| {
        if let Some(existing) = component.on_unmount.take() {
            component.on_unmount = Some(Box::new(move || {
                existing();
                f();
            }));
        } else {
            component.on_unmount = Some(Box::new(f));
        }
    });
}
