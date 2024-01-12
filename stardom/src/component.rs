use std::{cell::RefCell, thread_local};

use crate::{
    node::{Node, NodeKind},
    reactive::{untrack, ItemKey, Runtime},
};

thread_local! {
    static STACK: RefCell<Vec<Component>> = RefCell::default();
}

type UnitFn = Box<dyn FnMut()>;

pub(crate) struct Component {
    pub did_mount: bool,
    pub on_mount: Option<UnitFn>,
    pub on_unmount: Option<UnitFn>,

    rt: &'static Runtime,
    items: Vec<ItemKey>,
}

impl Component {
    pub fn add(&mut self, key: ItemKey) {
        self.items.push(key);
    }
}

impl Drop for Component {
    fn drop(&mut self) {
        if let Some(on_unmount) = &mut self.on_unmount {
            on_unmount();
        }

        let mut items = self.rt.items.borrow_mut();
        for key in &self.items {
            items.remove(*key);
        }
    }
}

pub(crate) fn create_component<F>(f: F) -> Node
where
    F: FnOnce() -> Node,
{
    let rt = Runtime::unwrap();
    STACK.with_borrow_mut(|stack| {
        stack.push(Component {
            did_mount: false,
            on_mount: None,
            on_unmount: None,
            rt,
            items: vec![],
        })
    });
    let content = untrack(f);
    let component = STACK.with_borrow_mut(|stack| stack.pop().unwrap());
    let node = Node::new(NodeKind::Component(component));
    node.insert(&content, None);
    node
}

pub(crate) fn with_active<U, F>(f: F) -> Option<U>
where
    F: FnOnce(&mut Component) -> U,
{
    STACK.with_borrow_mut(|stack| stack.last_mut().map(f))
}

pub fn on_mount<F: FnMut() + 'static>(f: F) {
    STACK.with_borrow_mut(|stack| {
        let component = stack.last_mut().expect("not within a component");
        if component.on_mount.replace(Box::new(f)).is_some() {
            panic!("already called on_mount within component");
        }
    });
}

pub fn on_unmount<F: FnMut() + 'static>(f: F) {
    STACK.with_borrow_mut(|stack| {
        let component = stack.last_mut().expect("not within a component");
        if component.on_unmount.replace(Box::new(f)).is_some() {
            panic!("already called on_unmount within component");
        }
    })
}
