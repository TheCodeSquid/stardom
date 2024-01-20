// TODO: try_* variants of the node operations for error handling

use std::{
    cell::{Cell, RefCell},
    rc::{Rc, Weak},
};

use indexmap::IndexMap;
use wasm_bindgen::{intern, prelude::*};

use crate::{
    component::{create_component, Component},
    event::{EventKey, EventOptions},
    web::{document, is_web},
};

pub use stardom_macros::{component, element, fragment};

type EventClosure = Closure<dyn Fn(web_sys::Event)>;

#[derive(Clone)]
pub struct Node(Rc<RawNode>);

impl Node {
    pub(crate) fn new(kind: NodeKind) -> Self {
        let native = document().and_then(|document| match &kind {
            NodeKind::Element {
                namespace,
                name,
                attrs,
            } => {
                intern(name);
                let element = if let Some(ns) = namespace {
                    intern(ns);
                    document.create_element_ns(namespace.as_deref(), name)
                } else {
                    document.create_element(name)
                }
                .unwrap();

                for (key, value) in &*attrs.borrow() {
                    element.set_attribute(key, value).unwrap();
                }

                Some(element.unchecked_into())
            }
            NodeKind::Text(content) => {
                let node = web_sys::Text::new_with_data(&content.borrow())
                    .unwrap()
                    .unchecked_into();
                Some(node)
            }
            NodeKind::Raw(_) | NodeKind::Component(_) | NodeKind::Fragment => None,
        });

        let raw = RawNode::new(kind, native);
        Self(Rc::new(raw))
    }

    pub(crate) fn from_element(element: web_sys::Element) -> Self {
        let mut attrs = IndexMap::new();
        let native_attrs = element.attributes();
        for i in 0..native_attrs.length() {
            let attr = native_attrs.item(i).unwrap();
            attrs.insert(attr.name(), attr.value());
        }

        let kind = NodeKind::Element {
            namespace: element.namespace_uri(),
            name: element.local_name(),
            attrs: RefCell::new(attrs),
        };

        let raw = RawNode::new(kind, Some(element.unchecked_into()));
        Self(Rc::new(raw))
    }

    pub fn element(name: impl Into<String>) -> Self {
        let kind = NodeKind::Element {
            namespace: None,
            name: name.into(),
            attrs: RefCell::default(),
        };
        Self::new(kind)
    }

    pub fn element_ns(namespace: impl Into<String>, name: impl Into<String>) -> Self {
        let kind = NodeKind::Element {
            namespace: Some(namespace.into()),
            name: name.into(),
            attrs: RefCell::default(),
        };
        Self::new(kind)
    }

    pub fn text(content: impl Into<String>) -> Self {
        let kind = NodeKind::Text(RefCell::new(content.into()));
        Self::new(kind)
    }

    pub fn raw(content: impl Into<String>) -> Self {
        let kind = NodeKind::Raw(RefCell::default());
        let node = Self::new(kind);
        node.set_text(content);
        node
    }

    pub fn component<F>(f: F) -> Self
    where
        F: FnOnce() -> Self,
    {
        create_component(f)
    }

    pub fn fragment() -> Self {
        Self::new(NodeKind::Fragment)
    }

    pub fn parent(&self) -> Option<Self> {
        self.0.parent.borrow().as_ref().and_then(Self::upgrade)
    }

    pub fn native(&self) -> Option<web_sys::Node> {
        self.0.native.clone()
    }

    pub fn insert(&self, child: &Self, before: Option<&Self>) {
        if let Some(parent) = child.parent() {
            parent.remove(child);
        }

        child.0.parent.replace(Some(self.downgrade()));
        if self.is_main() {
            child.mark_main();
        }

        {
            let mut children = self.0.children.borrow_mut();
            let index = if let Some(before) = before {
                children
                    .iter()
                    .position(|node| node == before)
                    .expect("insertion target not a child of self")
            } else {
                children.len()
            };

            children.insert(index, child.clone());

            // point self to next
            child.0.next.replace(children.get(index + 1).cloned());
            // point prev to self
            if let Some(prev) = index.checked_sub(1).and_then(|i| children.get(i)) {
                prev.0.next.replace(Some(child.clone()));
            }
        }

        if let Some(target) = self.native_insert_target() {
            let before = before.and_then(Self::native_prepend_target);
            child.mount(&target, before.as_ref());
        }
    }

    pub fn replace(&self, old: &Self, new: &Self) {
        if let Some(parent) = new.parent() {
            parent.remove(new);
        }

        old.0.parent.replace(None);
        new.0.parent.replace(Some(self.downgrade()));
        if self.is_main() {
            old.clear_main();
            new.mark_main();
        }

        {
            let mut children = self.0.children.borrow_mut();
            let index = children
                .iter()
                .position(|node| node == old)
                .expect("replacement target not a child of self");
            children[index] = new.clone();

            // point old to nothing
            old.0.next.replace(None);
            // point new to next
            new.0.next.replace(children.get(index + 1).cloned());
            // point prev to new
            if let Some(prev) = index.checked_sub(1).and_then(|i| children.get(i)) {
                prev.0.next.replace(Some(new.clone()));
            }
        }

        if let Some(target) = self.native_insert_target() {
            old.unmount(&target);

            let before = new.native_sibling();
            new.mount(&target, before.as_ref());
        }
    }

    pub fn remove(&self, child: &Self) {
        child.0.parent.replace(None);
        child.clear_main();

        {
            let mut children = self.0.children.borrow_mut();
            let index = children
                .iter()
                .position(|node| node == child)
                .expect("removal target not a child of self");
            children.remove(index);

            // point child to nothing
            child.0.next.replace(None);
            // point prev to next
            if let Some(prev) = index.checked_sub(1).and_then(|i| children.get(i)) {
                prev.0.next.replace(children.get(index + 1).cloned());
            }
        }

        if let Some(target) = self.native_insert_target() {
            child.unmount(&target);
        }
    }

    pub fn set_text(&self, content: impl Into<String>) {
        match &self.0.kind {
            NodeKind::Text(current) => {
                let content = content.into();

                if let Some(native) = &self.0.native {
                    native.set_text_content(Some(&content))
                }

                current.replace(content);
            }
            NodeKind::Raw(current) => {
                let content = content.into();

                if is_web() {
                    let children = self.0.children.borrow().clone();
                    for child in children {
                        self.remove(&child);
                    }
                    insert_raw_children(self, &content);
                }

                current.replace(content);
            }
            _ => panic!("can only set text content of text and raw nodes"),
        }
    }

    pub fn set_attr(&self, key: impl Into<String>, value: impl Into<String>) {
        if let NodeKind::Element { attrs, .. } = &self.0.kind {
            let key = key.into();
            intern(&key);
            let value = value.into();

            if let Some(native) = &self.0.native {
                native
                    .unchecked_ref::<web_sys::Element>()
                    .set_attribute(&key, &value)
                    .unwrap();
            }

            attrs.borrow_mut().insert(key, value);
        } else {
            panic!("can only set attributes on element nodes");
        }
    }

    pub fn remove_attr(&self, key: impl Into<String>) {
        if let NodeKind::Element { attrs, .. } = &self.0.kind {
            let key = key.into();
            intern(&key);

            if let Some(native) = &self.0.native {
                native
                    .unchecked_ref::<web_sys::Element>()
                    .remove_attribute(&key)
                    .unwrap();
            }

            attrs.borrow_mut().remove(&key);
        } else {
            panic!("can only set attributes on element nodes");
        }
    }

    pub fn event<K, F>(&self, key: &K, opts: EventOptions, f: F)
    where
        K: EventKey,
        F: Fn(K::Event) + 'static,
    {
        if matches!(self.0.kind, NodeKind::Element { .. }) {
            if let Some(native) = &self.0.native {
                let ev_f = move |ev: web_sys::Event| {
                    f(ev.dyn_into().expect("event type mismatch"));
                };

                let closure = Rc::new_cyclic(|weak| {
                    EventClosure::wrap(if opts.once.unwrap_or(false) {
                        let node = self.clone();
                        let weak = weak.clone();
                        let f = move |ev| {
                            ev_f(ev);

                            let mut events = node.0.events.borrow_mut();
                            let index = events
                                .iter()
                                .position(|event| Rc::as_ptr(event) == weak.as_ptr())
                                .expect("once event removed early");
                            events.swap_remove(index);
                        };
                        Box::new(f)
                    } else {
                        Box::new(ev_f)
                    })
                });

                native
                    .add_event_listener_with_callback_and_add_event_listener_options(
                        key.name(),
                        closure.as_ref().as_ref().unchecked_ref(),
                        &opts.into(),
                    )
                    .unwrap();

                self.0.events.borrow_mut().push(closure);
            }
        } else {
            panic!("can only register events to element nodes");
        }
    }

    fn downgrade(&self) -> Weak<RawNode> {
        Rc::downgrade(&self.0)
    }

    fn upgrade(weak: &Weak<RawNode>) -> Option<Self> {
        weak.upgrade().map(Self)
    }

    fn mount(&self, target: &web_sys::Node, before: Option<&web_sys::Node>) {
        if let Some(native) = &self.0.native {
            target.insert_before(native, before).unwrap();
        } else {
            for child in &*self.0.children.borrow() {
                child.mount(target, before);
            }
        }
    }

    fn unmount(&self, target: &web_sys::Node) {
        if let Some(native) = &self.0.native {
            target.remove_child(native).unwrap();
        } else {
            for child in &*self.0.children.borrow() {
                child.unmount(target);
            }
        }
    }

    fn is_main(&self) -> bool {
        self.0.main_tree.get()
    }

    pub(crate) fn mark_main(&self) {
        if !is_web() || self.is_main() {
            return;
        }

        self.0.main_tree.set(true);
        for child in &*self.0.children.borrow() {
            child.mark_main();
        }

        if let NodeKind::Component(Component {
            on_mount: Some(on_mount),
            ..
        }) = &self.0.kind
        {
            on_mount();
        }
    }

    pub(crate) fn clear_main(&self) {
        if !is_web() || !self.is_main() {
            return;
        }

        self.0.main_tree.set(false);
        for child in &*self.0.children.borrow() {
            child.clear_main();
        }

        if let NodeKind::Component(Component {
            on_unmount: Some(on_unmount),
            ..
        }) = &self.0.kind
        {
            on_unmount();
        }
    }

    /// Returns the nearest ancestor node bound to a native node, *including* `self`.
    fn native_insert_target(&self) -> Option<web_sys::Node> {
        if !is_web() {
            return None;
        }

        self.native().or_else(|| {
            self.parent()
                .and_then(|parent| parent.native().or_else(|| parent.native_insert_target()))
        })
    }

    /// Returns the topmost native node relative to *but excluding* `self`.
    fn native_sibling(&self) -> Option<web_sys::Node> {
        let next = self
            .0
            .next
            .borrow()
            .as_ref()
            .and_then(Self::native_prepend_target);
        if next.is_some() {
            return next;
        }

        self.parent().and_then(|parent| {
            if parent.0.native.is_none() {
                parent.native_sibling()
            } else {
                None
            }
        })
    }

    /// Returns the topmost native node relative to *and including* `self`.
    fn native_prepend_target(&self) -> Option<web_sys::Node> {
        // Returns itself if not virtual
        self.native()
            // otherwise, try the first child with a native prepend target
            .or_else(|| {
                self.0
                    .children
                    .borrow()
                    .iter()
                    .find_map(Self::native_prepend_target)
            })
            // otherwise, try the next sibling
            .or_else(|| {
                self.0
                    .next
                    .borrow()
                    .as_ref()
                    .and_then(Self::native_prepend_target)
            })
            // otherwise, try the parent's next sibling ONLY when the parent is virtual
            .or_else(|| {
                self.parent().and_then(|parent| {
                    if parent.0.native.is_none() {
                        parent
                            .0
                            .next
                            .borrow()
                            .as_ref()
                            .and_then(Self::native_prepend_target)
                    } else {
                        None
                    }
                })
            })
    }
}

impl Eq for Node {}
impl PartialEq for Node {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }
}

impl From<&str> for Node {
    fn from(value: &str) -> Self {
        Self::text(value)
    }
}

impl From<String> for Node {
    fn from(value: String) -> Self {
        Self::text(value)
    }
}

impl From<Vec<Self>> for Node {
    fn from(value: Vec<Self>) -> Self {
        let frag = Self::fragment();
        for node in &value {
            frag.insert(node, None);
        }
        frag
    }
}

struct RawNode {
    kind: NodeKind,

    parent: RefCell<Option<Weak<RawNode>>>,
    next: RefCell<Option<Node>>,
    children: RefCell<Vec<Node>>,

    main_tree: Cell<bool>,
    native: Option<web_sys::Node>,
    events: RefCell<Vec<Rc<EventClosure>>>,
}

impl RawNode {
    fn new(kind: NodeKind, native: Option<web_sys::Node>) -> Self {
        Self {
            kind,
            parent: RefCell::default(),
            next: RefCell::default(),
            children: RefCell::default(),
            main_tree: Cell::new(false),
            native,
            events: RefCell::default(),
        }
    }
}

pub(crate) enum NodeKind {
    Element {
        namespace: Option<String>,
        name: String,
        attrs: RefCell<IndexMap<String, String>>,
    },
    Text(RefCell<String>),
    Raw(RefCell<String>),
    Component(Component),
    Fragment,
}

fn insert_raw_children(target: &Node, raw: &str) {
    let children = parse_raw_html(raw);
    for i in 0..children.length() {
        let native = children.get(i).unwrap();
        let mut holder = Node::fragment();
        Rc::get_mut(&mut holder.0).unwrap().native = Some(native);
        target.insert(&holder, None);
    }
}

fn parse_raw_html(raw: &str) -> web_sys::NodeList {
    let range = web_sys::Range::new().unwrap();
    let doc = range.create_contextual_fragment(raw).unwrap();
    doc.child_nodes()
}

#[cfg(all(test, target_family = "wasm"))]
mod wasm_tests {
    use wasm_bindgen_test::*;

    use super::*;

    #[wasm_bindgen_test]
    fn native_node_creation() {
        let element = Node::element("div");
        assert!(element.native().is_some());

        let text = Node::text("Some random text");
        assert!(text.native().is_some());

        let fragment = Node::fragment();
        assert!(fragment.native().is_none());
    }

    #[wasm_bindgen_test]
    fn native_node_reordering() {
        let root = Node::element("div");
        let root_native = root.native().unwrap();

        let virtual1 = Node::fragment();
        let virtual2 = Node::fragment();
        let virtual3 = Node::fragment();

        let a = Node::text("a");
        let b = Node::text("b");
        let c = Node::text("c");
        let a_native = a.native().unwrap();
        let b_native = b.native().unwrap();
        let c_native = c.native().unwrap();

        virtual1.insert(&a, None);
        virtual2.insert(&virtual3, None);
        virtual3.insert(&b, None);

        root.insert(&virtual1, None);
        root.insert(&virtual2, Some(&virtual1));

        assert_eq!(root_native.child_nodes().length(), 2);
        assert_eq!(b_native.next_sibling().as_ref(), Some(&a_native));

        virtual3.replace(&b, &c);
        assert_eq!(c_native.next_sibling().as_ref(), Some(&a_native));
    }

    #[wasm_bindgen_test]
    fn raw_node_children() {
        let source = "<div></div> Some text <!-- and a comment too -->";

        let raw = Node::raw(source);
        let children = raw.0.children.borrow();
        assert_eq!(children.len(), 3);

        let natives: Vec<_> = children.iter().map(|node| node.native().unwrap()).collect();
        assert!(natives[0].has_type::<web_sys::Element>());
        assert!(natives[1].has_type::<web_sys::Text>());
        assert!(natives[2].has_type::<web_sys::Comment>());
        assert_eq!(natives.get(3), None);

        drop(children);

        raw.set_text("<br>");
        assert_eq!(raw.0.children.borrow().len(), 1);
    }
}
