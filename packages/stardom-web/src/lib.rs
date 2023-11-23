use std::{
    cell::RefCell,
    rc::{Rc, Weak},
    thread_local,
};

use stardom_nodes::{EventKey, Node, NodeType};
use wasm_bindgen::{intern, prelude::*};

thread_local! {
    static WINDOW: web_sys::Window = web_sys::window().unwrap();

    static DOCUMENT: web_sys::Document = web_sys::window()
        .unwrap()
        .document()
        .unwrap();
}

pub fn window() -> web_sys::Window {
    WINDOW.with(Clone::clone)
}

pub fn document() -> web_sys::Document {
    DOCUMENT.with(Clone::clone)
}

#[derive(Clone, Debug)]
pub struct DomNode(Rc<Inner>);

type WeakNode = Weak<Inner>;
type EventClosure = Closure<dyn Fn(web_sys::Event)>;

#[derive(Debug)]
struct Inner {
    native: web_sys::Node,
    ty: NodeType,

    parent: RefCell<Option<WeakNode>>,
    children: RefCell<Vec<DomNode>>,
    events: RefCell<Vec<EventClosure>>,
}

impl DomNode {
    fn new(native: web_sys::Node, ty: NodeType) -> Self {
        Self(Rc::new(Inner {
            native,
            ty,
            parent: RefCell::default(),
            children: RefCell::default(),
            events: RefCell::default(),
        }))
    }

    pub fn native(&self) -> &web_sys::Node {
        &self.0.native
    }

    fn is_virtual(&self) -> bool {
        matches!(self.ty(), NodeType::Fragment | NodeType::Raw)
    }

    fn from_native(native: web_sys::Node) -> Option<Self> {
        let ty = if native.has_type::<web_sys::Element>() {
            NodeType::Element
        } else if native.has_type::<web_sys::Text>() {
            NodeType::Text
        } else if native.has_type::<web_sys::Comment>() {
            NodeType::Fragment
        } else {
            return None;
        };

        Some(Self::new(native, ty))
    }

    fn native_parent(&self) -> Option<web_sys::Node> {
        self.0.native.parent_node()
    }

    fn native_target(&self) -> Option<web_sys::Node> {
        if self.is_virtual() {
            self.native_parent()
        } else {
            Some(self.0.native.clone())
        }
    }

    fn first_node(&self) -> web_sys::Node {
        if self.is_virtual() {
            let children = self.0.children.borrow();
            if let Some(first) = children.first() {
                return first.0.native.clone();
            }
        }

        self.0.native.clone()
    }

    pub fn mount_to_native(&self, target: &web_sys::Node, before: Option<&web_sys::Node>) {
        if self.is_virtual() {
            let children = self.0.children.borrow();
            for child in &*children {
                child.mount_to_native(target, before);
            }
        }

        target.insert_before(&self.0.native, before).unwrap();
    }

    pub fn remove_from_native(&self, target: &web_sys::Node) {
        if self.is_virtual() {
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

        Self::new(native.unchecked_into(), NodeType::Element)
    }

    fn text() -> Self {
        let native = web_sys::Text::new().unwrap();

        Self::new(native.unchecked_into(), NodeType::Text)
    }

    fn fragment() -> Self {
        let native = web_sys::Comment::new().unwrap();

        Self::new(native.unchecked_into(), NodeType::Fragment)
    }

    fn raw() -> Self {
        let native = web_sys::Comment::new().unwrap();

        Self::new(native.unchecked_into(), NodeType::Raw)
    }

    fn ty(&self) -> NodeType {
        self.0.ty
    }

    fn parent(&self) -> Option<Self> {
        self.0
            .parent
            .borrow()
            .as_ref()
            .and_then(Weak::upgrade)
            .map(DomNode)
    }

    fn children(&self) -> Vec<Self> {
        self.0.children.borrow().clone()
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

        child.0.parent.borrow_mut().replace(Rc::downgrade(&self.0));

        if let Some(target) = self.native_target() {
            let before = before.map(|node| node.first_node()).or_else(|| {
                if self.is_virtual() {
                    self.native().next_sibling()
                } else {
                    None
                }
            });

            child.mount_to_native(&target, before.as_ref());
        }
    }

    fn remove(&self, child: &Self) {
        let mut children = self.0.children.borrow_mut();
        let idx = children
            .iter()
            .position(|node| node == child)
            .expect("not a parent of child node");
        children.remove(idx);

        child.0.parent.borrow_mut().take();

        if let Some(target) = self.native_target() {
            child.remove_from_native(&target);
        }
    }

    fn set_text(&self, content: &str) {
        match self.ty() {
            NodeType::Text => {
                self.0.native.set_text_content(Some(content));
            }
            NodeType::Raw => {
                for child in self.0.children.borrow().clone() {
                    self.remove(&child);
                }

                let range = web_sys::Range::new().unwrap();
                let doc = range.create_contextual_fragment(content).unwrap();
                let native_nodes = doc.child_nodes();

                for i in 0..native_nodes.length() {
                    let native = native_nodes.get(i).unwrap();
                    if let Some(node) = Self::from_native(native) {
                        self.insert(&node, None);
                    }
                }
            }
            _ => panic!("can only set text content of text or raw nodes"),
        }
    }

    fn attr(&self, name: &str) -> Option<String> {
        if self.ty() == NodeType::Element {
            self.0
                .native
                .unchecked_ref::<web_sys::Element>()
                .get_attribute(name)
        } else {
            panic!("attributes only exist on element nodes");
        }
    }

    fn set_attr(&self, name: &str, value: &str) {
        if self.ty() == NodeType::Element {
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
        if self.ty() == NodeType::Element {
            self.0
                .native
                .unchecked_ref::<web_sys::Element>()
                .remove_attribute(intern(name))
                .unwrap();
        } else {
            panic!("attributes only exist on element nodes");
        }
    }

    fn event<E, F>(&self, event: &E, f: F)
    where
        E: EventKey,
        F: Fn(E::Event) + 'static,
    {
        if self.ty() != NodeType::Element {
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

#[cfg(all(test, target_family = "wasm"))]
mod tests {
    use crate::{DomNode as N, Node};
    use wasm_bindgen_test::*;

    wasm_bindgen_test_configure!(run_in_browser);

    #[wasm_bindgen_test]
    fn fragment_insertion() {
        let root = N::element(None, "div");

        let last = N::text();
        root.insert(&last, None);

        let outer = N::fragment();
        let inner = N::text();

        root.insert(&outer, Some(&last));
        outer.insert(&inner, None);

        assert_eq!(inner.native().next_sibling(), Some(last.native().clone()));
    }
}
