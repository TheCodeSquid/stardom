use std::{cell::RefCell, rc::Rc, thread_local};

use crate::{
    node::{Node, NodeKind},
    reactive::Item,
};

thread_local! {
    static STACK: RefCell<Vec<Component>> = RefCell::default();
}

type UnitFn = Box<dyn Fn()>;

#[derive(Default)]
pub(crate) struct Component {
    pub on_mount: Option<UnitFn>,
    pub on_unmount: Option<UnitFn>,

    pub _items: Vec<Rc<Item>>,
}

pub(crate) fn create_component<F>(f: F) -> Node
where
    F: FnOnce() -> Node,
{
    STACK.with_borrow_mut(|stack| stack.push(Component::default()));
    let content = f();
    let component = STACK.with_borrow_mut(|stack| stack.pop().unwrap());
    let node = Node::new(NodeKind::Component(component));
    node.insert(&content, None);
    node
}

pub(crate) fn active<U, F>(f: F) -> Option<U>
where
    F: FnOnce(&mut Component) -> U,
{
    STACK.with_borrow_mut(|stack| stack.last_mut().map(f))
}

pub fn on_mount<F: Fn() + 'static>(f: F) {
    STACK.with_borrow_mut(|stack| {
        let component = stack.last_mut().expect("not within a component");
        if component.on_mount.replace(Box::new(f)).is_some() {
            panic!("already called on_mount within component");
        }
    });
}

pub fn on_unmount<F: Fn() + 'static>(f: F) {
    STACK.with_borrow_mut(|stack| {
        let component = stack.last_mut().expect("not within a component");
        if component.on_unmount.replace(Box::new(f)).is_some() {
            panic!("already called on_unmount within component");
        }
    })
}
