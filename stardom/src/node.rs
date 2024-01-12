use std::{
    cell::RefCell,
    rc::{Rc, Weak},
};

use indexmap::IndexMap;
use wasm_bindgen::{intern, prelude::*};

use crate::{
    component::{create_component, Component},
    reactive::effect,
    EventKey,
};

type EventClosure = Closure<dyn Fn(web_sys::Event)>;

#[derive(Clone)]
pub struct Node(pub(crate) Rc<Inner>);

#[derive(Clone)]
pub struct WeakNode(Weak<Inner>);

pub(crate) struct Inner {
    pub(crate) native: Option<web_sys::Node>,
    pub(crate) state: RefCell<State>,
}

pub(crate) struct State {
    pub(crate) kind: NodeKind,
    pub(crate) main_tree: bool,
    pub(crate) parent: Option<WeakNode>,
    pub(crate) next: Option<Node>,
    pub(crate) children: Vec<Node>,
    pub(crate) events: Vec<EventClosure>,
}

pub(crate) enum NodeKind {
    Text(String),
    Element(Element),
    Raw(String),
    Component(Component),
    Fragment,
}

pub(crate) struct Element {
    pub(crate) namespace: Option<String>,
    pub(crate) name: String,
    pub(crate) attrs: IndexMap<String, String>,
}

impl Node {
    pub(crate) fn new(kind: NodeKind) -> Self {
        let native = crate::document().and_then(|document| match &kind {
            NodeKind::Text(content) => Some(
                web_sys::Text::new_with_data(content)
                    .unwrap()
                    .unchecked_into(),
            ),
            NodeKind::Element(Element {
                namespace, name, ..
            }) => Some(
                if let Some(ns) = &namespace {
                    document
                        .create_element_ns(Some(intern(ns)), intern(name))
                        .unwrap()
                } else {
                    document.create_element(intern(name)).unwrap()
                }
                .unchecked_into(),
            ),
            NodeKind::Raw(_) => None,
            NodeKind::Fragment => None,
            NodeKind::Component(_) => None,
        });

        let state = State {
            kind,
            main_tree: false,
            parent: None,
            next: None,
            children: vec![],
            events: vec![],
        };
        let inner = Inner {
            native,
            state: RefCell::new(state),
        };
        Self(Rc::new(inner))
    }

    pub fn text<S: Into<String>>(content: S) -> Self {
        Self::new(NodeKind::Text(content.into()))
    }

    pub fn element(namespace: Option<String>, name: String) -> Self {
        Self::new(NodeKind::Element(Element {
            namespace,
            name,
            attrs: IndexMap::new(),
        }))
    }

    pub fn raw<S: Into<String>>(content: S) -> Self {
        let node = Self::new(NodeKind::Raw(String::new()));
        node.set_text(content.into());
        node
    }

    pub fn component<F: FnOnce() -> Self>(f: F) -> Self {
        create_component(f)
    }

    pub fn fragment() -> Self {
        Self::new(NodeKind::Fragment)
    }

    pub fn native(&self) -> Option<web_sys::Node> {
        self.0.native.clone()
    }

    pub fn downgrade(&self) -> WeakNode {
        WeakNode(Rc::downgrade(&self.0))
    }

    pub fn set_text<S: Into<String>>(&self, content: S) {
        let mut state = self.0.state.borrow_mut();
        match &mut state.kind {
            NodeKind::Text(current) => {
                *current = content.into();

                if let Some(native) = self.native() {
                    native.set_text_content(Some(current));
                }
            }
            NodeKind::Raw(current) => {
                *current = content.into();
                let current = current.clone();

                if let Some(document) = crate::document() {
                    let children = std::mem::take(&mut state.children);
                    drop(state);
                    for child in children {
                        self.remove(&child);
                    }

                    let range = document.create_range().unwrap();
                    let fragment = range.create_contextual_fragment(&current).unwrap();
                    let children = fragment.child_nodes();

                    for i in 0..children.length() {
                        let child = children.get(i).unwrap();
                        let mut node = Self::new(NodeKind::Fragment);
                        Rc::get_mut(&mut node.0).unwrap().native = Some(child);
                        self.insert(&node, None);
                    }
                }
            }
            _ => panic!("not a textual node"),
        }
    }

    pub fn set_attr(&self, key: String, value: String) {
        let mut state = self.0.state.borrow_mut();
        if let NodeKind::Element(element) = &mut state.kind {
            if let Some(native) = self.native() {
                native
                    .unchecked_ref::<web_sys::Element>()
                    .set_attribute(intern(&key), &value)
                    .unwrap();
            }

            element.attrs.insert(key, value);
        } else {
            panic!("attributes can only be set on elements");
        }
    }

    pub fn insert(&self, child: &Self, before: Option<&Self>) {
        let mut state = self.0.state.borrow_mut();
        if !state.kind.is_container() {
            panic!("not a container");
        }

        // Virtual
        let index = if let Some(before) = &before {
            state
                .children
                .iter()
                .position(|node| node == *before)
                .expect("prepend target not a child of parent")
        } else {
            state.children.len()
        };
        state.children.insert(index, child.clone());
        if state.main_tree {
            child.mark_main();
        }

        if index > 0 {
            if let Some(prev) = state.children.get(index - 1).cloned() {
                prev.0.state.borrow_mut().next = Some(child.clone());
            }
        }
        let next = state.children.get(index + 1).cloned();
        {
            let mut child = child.0.state.borrow_mut();
            child.parent = Some(self.downgrade());
            child.next = next;
        }

        // Native
        if crate::document().is_some() {
            let parent = self.native().or_else(|| {
                state
                    .parent
                    .as_ref()
                    .and_then(WeakNode::upgrade)
                    .as_ref()
                    .and_then(Self::native)
            });
            if let Some(parent) = parent {
                let before = before.and_then(Self::native_prepend_target);
                child.mount(parent.as_ref(), before.as_ref());
            }
        }
    }

    pub fn replace(&self, old: &Self, new: &Self) {
        let mut state = self.0.state.borrow_mut();
        if !state.kind.is_container() {
            panic!("not a container");
        }

        if old == new {
            return;
        }

        // Virtual
        let index = state
            .children
            .iter()
            .position(|node| node == old)
            .expect("replacement target not a child of parent");
        state.children[index] = new.clone();
        old.clear_main();
        if state.main_tree {
            new.mark_main();
        } else {
            new.clear_main();
        }
        if index > 0 {
            if let Some(prev) = state.children.get(index - 1) {
                prev.0.state.borrow_mut().next = Some(new.clone());
            }
        };
        let next = state.children.get(index + 1).cloned();
        {
            let mut old = old.0.state.borrow_mut();
            old.parent = None;
            old.next = None;
            let mut new = new.0.state.borrow_mut();
            new.parent = Some(self.downgrade());
            new.next = next.clone();
        }

        // Native
        if crate::document().is_some() {
            let parent = self.native().or_else(|| {
                state
                    .parent
                    .as_ref()
                    .and_then(WeakNode::upgrade)
                    .as_ref()
                    .and_then(Self::native)
            });
            if let Some(parent) = parent {
                old.unmount(&parent);
                let before = next.as_ref().and_then(Self::native_prepend_target);
                new.mount(&parent, before.as_ref());
            }
        }
    }

    pub fn remove(&self, child: &Self) {
        let mut state = self.0.state.borrow_mut();
        if !state.kind.is_container() {
            panic!("not a container");
        }

        // Virtual
        child.clear_main();
        let index = state
            .children
            .iter()
            .position(|node| node == child)
            .expect("removal target not a child of parent");
        state.children.remove(index);

        {
            let mut child = child.0.state.borrow_mut();
            child.parent = None;
            child.next = None;
        }
        let next = state.children.get(index);
        if index > 0 {
            if let Some(prev) = state.children.get(index - 1) {
                prev.0.state.borrow_mut().next = next.cloned();
            }
        }

        // Native
        if crate::document().is_some() {
            let parent = self.native().or_else(|| {
                state
                    .parent
                    .as_ref()
                    .and_then(WeakNode::upgrade)
                    .as_ref()
                    .and_then(Self::native)
            });
            if let Some(parent) = parent {
                child.unmount(parent.as_ref());
            }
        }
    }

    pub fn event<E, F>(&self, key: E, passive: bool, f: F)
    where
        E: EventKey,
        F: Fn(E::Value) + 'static,
    {
        if let Some(native) = self.native() {
            let name = key.name();
            let closure = EventClosure::new(move |ev: web_sys::Event| {
                f(ev.unchecked_into());
            });

            let mut opts = web_sys::AddEventListenerOptions::new();
            opts.passive(passive);
            native
                .add_event_listener_with_callback_and_add_event_listener_options(
                    name,
                    closure.as_ref().unchecked_ref(),
                    &opts,
                )
                .unwrap();

            self.0.state.borrow_mut().events.push(closure);
        }
    }

    pub(crate) fn mark_main(&self) {
        let mut state = self.0.state.borrow_mut();
        if state.main_tree {
            return;
        }

        state.main_tree = true;
        for child in &state.children {
            child.mark_main();
        }

        if let NodeKind::Component(component) = &mut state.kind {
            if component.did_mount {
                return;
            }
            component.did_mount = true;

            if let Some(on_mount) = &mut component.on_mount {
                on_mount();
            }
        }
    }

    fn clear_main(&self) {
        let mut state = self.0.state.borrow_mut();
        if !state.main_tree {
            return;
        }

        state.main_tree = false;
        for child in &state.children {
            child.clear_main();
        }
    }

    pub(crate) fn mount(&self, native: &web_sys::Node, before: Option<&web_sys::Node>) {
        let state = self.0.state.borrow();
        if let Some(this) = self.native() {
            native.insert_before(&this, before).unwrap();
        } else {
            for child in &state.children {
                child.mount(native, before);
            }
        }
    }

    fn unmount(&self, native: &web_sys::Node) {
        let state = self.0.state.borrow();
        if let Some(this) = self.native() {
            native.remove_child(&this).unwrap();
        } else {
            for child in &state.children {
                child.unmount(native);
            }
        }
    }

    fn native_prepend_target(&self) -> Option<web_sys::Node> {
        let state = self.0.state.borrow();
        self.native()
            .or_else(|| state.children.last().and_then(Self::native_prepend_target))
            .or_else(|| {
                state
                    .next
                    .as_ref()
                    .and_then(|next| next.native_prepend_target())
            })
    }
}

impl From<String> for Node {
    fn from(value: String) -> Self {
        Self::text(value)
    }
}

impl From<&str> for Node {
    fn from(value: &str) -> Self {
        Self::text(value)
    }
}

impl WeakNode {
    pub fn upgrade(&self) -> Option<Node> {
        self.0.upgrade().map(Node)
    }
}

impl NodeKind {
    fn is_container(&self) -> bool {
        // should this account for void elements?
        !matches!(self, Self::Text(_))
    }
}

impl Eq for Node {}
impl PartialEq for Node {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }
}

impl Eq for WeakNode {}
impl PartialEq for WeakNode {
    fn eq(&self, other: &Self) -> bool {
        Weak::ptr_eq(&self.0, &other.0)
    }
}

pub trait CaptureNode: Sized {
    fn capture_node<F>(parent: &Node, f: F)
    where
        F: Fn() -> Self + 'static;
}

impl CaptureNode for Node {
    fn capture_node<F>(parent: &Node, f: F)
    where
        F: Fn() -> Self + 'static,
    {
        let mut current = Self::fragment();
        parent.insert(&current, None);
        effect({
            let parent = parent.clone();
            move || {
                let node = f();
                parent.replace(&current, &node);
                current = node;
            }
        });
    }
}

impl CaptureNode for Option<Node> {
    fn capture_node<F>(parent: &Node, f: F)
    where
        F: Fn() -> Self + 'static,
    {
        let mut current = Node::fragment();
        parent.insert(&current, None);
        effect({
            let parent = parent.clone();
            move || {
                let node = f().unwrap_or_else(Node::fragment);
                parent.replace(&current, &node);
                current = node;
            }
        });
    }
}

impl CaptureNode for &str {
    fn capture_node<F>(parent: &Node, f: F)
    where
        F: Fn() -> Self + 'static,
    {
        let node = Node::text("");
        parent.insert(&node, None);
        effect({
            let node = node.clone();
            move || node.set_text(f())
        });
    }
}

impl CaptureNode for String {
    fn capture_node<F>(parent: &Node, f: F)
    where
        F: Fn() -> Self + 'static,
    {
        let node = Node::text("");
        parent.insert(&node, None);
        effect({
            let node = node.clone();
            move || node.set_text(f())
        });
    }
}