use std::{
    cell::RefCell,
    fmt, mem,
    rc::{Rc, Weak},
    thread_local,
};

use bitflags::bitflags;
use stardom_nodes::{EventKey, Node};
use wasm_bindgen::{intern, prelude::*};

thread_local! {
    static DOCUMENT: web_sys::Document = web_sys::window()
        .unwrap()
        .document()
        .unwrap();
}

pub fn document() -> web_sys::Document {
    DOCUMENT.with(Clone::clone)
}

pub fn mount(node: DomNode, selector: &str) {
    let root = document()
        .query_selector(selector)
        .unwrap()
        .expect("node mount point not found");
    node.mount_to_native(&root, None);
    mem::forget(node);
}

bitflags! {
    #[derive(Clone, Copy, Debug)]
    struct Flags: u8 {
        const TEXT = 0b001;
        const FRAGMENT = 0b010;
        const RAW = 0b100;
    }
}

#[derive(Clone)]
pub struct DomNode(Rc<Inner>);

type WeakNode = Weak<Inner>;
type EventClosure = Closure<dyn Fn(web_sys::Event)>;

struct Inner {
    native: web_sys::Node,
    flags: Flags,

    parent: RefCell<Option<WeakNode>>,
    children: RefCell<Vec<DomNode>>,
    events: RefCell<Vec<EventClosure>>,
}

impl DomNode {
    fn new(native: web_sys::Node, flags: Flags) -> Self {
        Self(Rc::new(Inner {
            native,
            flags,
            parent: RefCell::default(),
            children: RefCell::default(),
            events: RefCell::default(),
        }))
    }

    pub fn native_parent(&self) -> Option<web_sys::Node> {
        self.0.native.parent_node()
    }

    pub fn native_target(&self) -> Option<web_sys::Node> {
        if self.0.flags.contains(Flags::FRAGMENT) {
            self.native_parent()
        } else {
            Some(self.0.native.clone())
        }
    }

    pub fn first_node(&self) -> web_sys::Node {
        if self.0.flags.contains(Flags::FRAGMENT) {
            let children = self.0.children.borrow();
            if let Some(first) = children.first() {
                return first.0.native.clone();
            }
        }

        self.0.native.clone()
    }

    pub fn mount_to_native(&self, target: &web_sys::Node, before: Option<&web_sys::Node>) {
        if self.0.flags.contains(Flags::FRAGMENT) {
            let children = self.0.children.borrow();
            for child in &*children {
                child.mount_to_native(target, before);
            }
        }

        target.insert_before(&self.0.native, before).unwrap();
    }

    pub fn remove_from_native(&self, target: &web_sys::Node) {
        if self.0.flags.contains(Flags::FRAGMENT) {
            let children = self.0.children.borrow();
            for child in &*children {
                child.remove_from_native(target);
            }
        }

        target.remove_child(&self.0.native).unwrap();
    }
}

impl Node for DomNode {
    fn element(namespace: Option<&str>, name: &str) -> Self {
        let native = DOCUMENT
            .with(|document| {
                if namespace.is_some() {
                    document.create_element_ns(namespace, name)
                } else {
                    document.create_element(name)
                }
            })
            .unwrap();

        Self::new(native.unchecked_into(), Flags::empty())
    }

    fn text() -> Self {
        let native = web_sys::Text::new().unwrap();

        Self::new(native.unchecked_into(), Flags::TEXT)
    }

    fn fragment() -> Self {
        let native = web_sys::Comment::new().unwrap();

        Self::new(native.unchecked_into(), Flags::FRAGMENT)
    }

    fn raw() -> Self {
        let native = web_sys::Comment::new().unwrap();

        Self::new(native.unchecked_into(), Flags::FRAGMENT | Flags::RAW)
    }

    fn parent(&self) -> Option<Self> {
        self.0
            .parent
            .borrow()
            .as_ref()
            .and_then(Weak::upgrade)
            .map(DomNode)
    }

    fn next_sibling(&self) -> Option<Self> {
        let parent = self.parent()?;
        let children = parent.0.children.borrow();
        children
            .iter()
            .position(|node| node == self)
            .and_then(|idx| children.get(idx + 1).cloned())
    }

    fn insert(&self, child: &Self, before: Option<&Self>) {
        let mut children = self.0.children.borrow_mut();
        let idx = if let Some(before) = before {
            children
                .iter()
                .position(|node| node == before)
                .expect("not a parent of insertion point node")
        } else {
            children.len()
        };
        children.insert(idx, child.clone());

        if let Some(target) = self.native_target() {
            child.mount_to_native(&target, before.map(|node| node.first_node()).as_ref());
        }
    }

    fn remove(&self, child: &Self) {
        let mut children = self.0.children.borrow_mut();
        if let Some(idx) = children.iter().position(|node| node == child) {
            children.remove(idx);
        }

        if let Some(target) = self.native_target() {
            child.remove_from_native(&target);
        }
    }

    fn set_attr(&self, name: &str, value: &str) {
        if self.0.flags.is_empty() {
            self.0
                .native
                .unchecked_ref::<web_sys::Element>()
                .set_attribute(intern(name), value)
                .unwrap();
        } else {
            panic!("attributes only exist on element nodes");
        }
    }

    fn remove_attr(&self, name: &str) {
        if self.0.flags.is_empty() {
            self.0
                .native
                .unchecked_ref::<web_sys::Element>()
                .remove_attribute(intern(name))
                .unwrap();
        } else {
            panic!("attributes only exist on element nodes");
        }
    }

    fn set_text(&self, content: &str) {
        if self.0.flags.contains(Flags::TEXT) {
            self.0.native.set_text_content(Some(content));
        } else if self.0.flags.contains(Flags::RAW) {
            // TODO: clear children here

            let range = web_sys::Range::new().unwrap();
            let doc = range.create_contextual_fragment(content).unwrap();
            let native_nodes = doc.child_nodes();

            for i in 0..native_nodes.length() {
                let native = native_nodes.get(i).unwrap();
                let holder = Self::new(native, Flags::empty());
                self.insert(&holder, None);
            }
        } else {
            panic!("can only set text content of text or raw nodes");
        }
    }

    fn event<E, F>(&self, event: &E, f: F)
    where
        E: EventKey,
        F: Fn(E::Event) + 'static,
    {
        if !self.0.flags.is_empty() {
            panic!("can only set events on element nodes");
        }

        let closure = EventClosure::new(move |value: web_sys::Event| {
            let value = value
                .dyn_into::<E::Event>()
                .expect("invalid event type cast");
            f(value);
        });

        self.0
            .native
            .add_event_listener_with_callback(
                intern(event.name()),
                closure.as_ref().unchecked_ref(),
            )
            .unwrap();

        self.0.events.borrow_mut().push(closure);
    }
}

impl PartialEq for DomNode {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }
}

impl Eq for DomNode {}

impl fmt::Debug for DomNode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("DomNode")
            .field("native", &self.0.native)
            .field("flags", &self.0.flags)
            .field("children", &self.0.children)
            .finish()
    }
}
