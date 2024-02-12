use std::{
    any::{self, Any, TypeId},
    cell::{Cell, RefCell},
    collections::HashMap,
    mem, thread_local,
};

use crate::{
    env::is_browser,
    node::{Node, NodeKind},
};

thread_local! {
    static STACK: RefCell<Vec<Component>> = RefCell::default();
}

#[derive(Default)]
pub(crate) struct Component {
    pub(crate) frozen: Cell<bool>,
    pub(crate) mounted: Cell<bool>,

    contexts: HashMap<TypeId, Box<dyn Any>>,
    pub(crate) on_mount: RefCell<Vec<Box<dyn FnOnce()>>>,
    pub(crate) on_unmount: RefCell<Vec<Box<dyn FnOnce()>>>,
}

impl Component {
    pub(crate) fn create<F>(f: F) -> Node
    where
        F: FnOnce() -> Node,
    {
        STACK.with_borrow_mut(|stack| stack.push(Self::default()));
        let content = f();
        let component = STACK.with_borrow_mut(|stack| stack.pop().unwrap());

        let node = Node::create(NodeKind::Component(component));
        node.insert(&content, None);
        node
    }

    pub(crate) fn on_mount(&self) {
        if !self.mounted.get() {
            self.mounted.set(true);
            for on_mount in mem::take(&mut *self.on_mount.borrow_mut()) {
                on_mount();
            }
        }
    }
}

impl Drop for Component {
    fn drop(&mut self) {
        if self.mounted.get() {
            for on_unmount in mem::take(&mut *self.on_unmount.borrow_mut()) {
                on_unmount();
            }
        }
    }
}

pub fn on_mount<F>(f: F)
where
    F: FnOnce() + 'static,
{
    if is_browser() {
        active(|component| {
            component.on_mount.borrow_mut().push(Box::new(f));
        });
    }
}

pub fn on_unmount<F>(f: F)
where
    F: FnOnce() + 'static,
{
    if is_browser() {
        active(|component| {
            component.on_unmount.borrow_mut().push(Box::new(f));
        });
    }
}

pub fn register_context<T: 'static>(context: T) {
    active(|component| {
        component
            .contexts
            .insert(context.type_id(), Box::new(context));
    })
}

pub fn try_with_context<T, U, F>(f: F) -> Option<U>
where
    T: 'static,
    F: FnOnce(&T) -> U,
{
    STACK.with_borrow(|stack| {
        stack
            .iter()
            .find_map(|component| component.contexts.get(&TypeId::of::<T>()))
            .map(|ctx| {
                f(ctx.downcast_ref::<T>().unwrap_or_else(|| {
                    panic!("context type mismatch (expected {})", any::type_name::<T>())
                }))
            })
    })
}

pub fn with_context<T, U, F>(f: F) -> U
where
    T: 'static,
    F: FnOnce(&T) -> U,
{
    try_with_context(f)
        .unwrap_or_else(|| panic!("context of type `{}` not registered", any::type_name::<T>()))
}

pub fn try_context<T>() -> Option<T>
where
    T: Clone + 'static,
{
    try_with_context(Clone::clone)
}

pub fn context<T>() -> T
where
    T: Clone + 'static,
{
    with_context(Clone::clone)
}

fn active<F>(f: F)
where
    F: FnOnce(&mut Component),
{
    STACK
        .try_with(|stack| {
            f(stack
                .borrow_mut()
                .last_mut()
                .expect("not invoked during component creation"))
        })
        .ok();
}
