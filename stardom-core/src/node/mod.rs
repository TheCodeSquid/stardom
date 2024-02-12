pub mod browser;
pub mod hydrate;
pub mod render;

use std::{
    cell::Cell,
    cell::{Ref, RefCell},
    collections::HashMap,
    rc::{Rc, Weak},
    thread_local,
};

use indexmap::IndexMap;
use wasm_bindgen::{intern, prelude::*};

use crate::{
    component::Component,
    env::{is_browser, is_hydrating},
    events::{EventKey, EventOptions},
    util::document,
};

type EventClosure = Closure<dyn FnMut(web_sys::Event)>;

pub(crate) enum NodeKind {
    Element {
        name: String,
        namespace: Option<String>,
        attrs: RefCell<IndexMap<String, String>>,
    },
    Text(RefCell<String>),
    Raw(RefCell<String>),
    Fragment,
    Component(Component),
}

impl NodeKind {
    fn create_native(&self) -> Option<web_sys::Node> {
        match self {
            Self::Element {
                name,
                namespace,
                attrs,
            } => {
                intern(name);
                let element = if let Some(ns) = namespace {
                    intern(ns);
                    document().create_element_ns(Some(ns), name)
                } else {
                    document().create_element(name)
                }
                .unwrap();

                for (key, value) in &*attrs.borrow() {
                    element.set_attribute(key, value).unwrap();
                }

                Some(element.unchecked_into())
            }
            Self::Text(value) => {
                let text = web_sys::Text::new_with_data(&value.borrow()).unwrap();
                Some(text.unchecked_into())
            }
            Self::Raw(_) | Self::Fragment | Self::Component(_) => None,
        }
    }
}

#[derive(Clone)]
pub struct Node(Rc<RawNode>);

struct RawNode {
    main_tree: Cell<bool>,
    kind: NodeKind,

    parent: RefCell<Option<Weak<RawNode>>>,
    next: RefCell<Option<Node>>,
    children: RefCell<Vec<Node>>,

    browser: Option<BrowserNode>,
}

#[derive(Default)]
struct BrowserNode {
    native: RefCell<Option<web_sys::Node>>,
    events: RefCell<HashMap<u64, EventEntry>>,
}

struct EventEntry {
    name: String,
    closure: EventClosure,
    options: EventOptions,
}

impl Node {
    // Node Creation //

    pub(crate) fn create(kind: NodeKind) -> Self {
        let browser = if is_browser() {
            let native = kind.create_native();
            Some(BrowserNode {
                native: RefCell::new(native),
                events: RefCell::default(),
            })
        } else if is_hydrating() {
            Some(BrowserNode::default())
        } else {
            None
        };

        let raw = RawNode {
            main_tree: Cell::new(false),
            kind,
            parent: RefCell::default(),
            next: RefCell::default(),
            children: RefCell::default(),
            browser,
        };
        Self(Rc::new(raw))
    }

    pub fn element(name: String) -> Self {
        let kind = NodeKind::Element {
            name,
            namespace: None,
            attrs: RefCell::default(),
        };
        Self::create(kind)
    }

    pub fn element_ns(name: String, namespace: String) -> Self {
        let kind = NodeKind::Element {
            name,
            namespace: Some(namespace),
            attrs: RefCell::default(),
        };
        Self::create(kind)
    }

    pub fn text(value: String) -> Self {
        let kind = NodeKind::Text(RefCell::new(value));
        Self::create(kind)
    }

    pub fn unsecure_raw(value: String) -> Self {
        let kind = NodeKind::Raw(RefCell::default());
        let node = Self::create(kind);
        node.set_raw_content(value);
        node
    }

    pub fn fragment() -> Self {
        Self::create(NodeKind::Fragment)
    }

    pub fn component<F>(f: F) -> Self
    where
        F: FnOnce() -> Self,
    {
        Component::create(f)
    }

    // Containers //

    pub fn insert(&self, child: &Self, before: Option<&Self>) {
        if let NodeKind::Component(component) = &self.0.kind {
            if !component.frozen.get() {
                component.frozen.set(true);
            } else {
                self.expect_kind(Expect::Container);
            }
        }

        if let Some(parent) = child.parent() {
            parent.remove(child);
        }

        child.set_main_tree(self.main_tree());

        {
            let mut children = self.0.children.borrow_mut();
            let index = if let Some(before) = before {
                children
                    .iter()
                    .position(|node| node == before)
                    .expect("prepend target not a child of self")
            } else {
                children.len()
            };
            children.insert(index, child.clone());

            child.0.parent.replace(Some(self.downgrade()));

            // point child to children[index + 1]
            child.0.next.replace(children.get(index + 1).cloned());

            // point children[index - 1] (if it exists) to child
            if let Some(prev) = index.checked_sub(1).and_then(|i| children.get(i)) {
                prev.0.next.replace(Some(child.clone()));
            }
        }

        if let Some(native) = self.to_native_anchor() {
            let before = before.and_then(Self::to_native_sibling);
            child.mount(&native, before.as_ref());
        }
    }

    pub fn remove(&self, child: &Self) {
        child.set_main_tree(false);

        {
            let mut children = self.0.children.borrow_mut();
            let index = children
                .iter()
                .position(|node| node == child)
                .expect("removal node not a child of self");

            child.0.parent.replace(None);
            child.0.next.replace(None);

            // point children[index - 1] to children[index + 1]
            if let Some(prev) = index.checked_sub(1).and_then(|i| children.get(i)) {
                prev.0.next.replace(children.get(index + 1).cloned());
            }

            children.remove(index);
        }

        if let Some(native) = self.to_native_anchor() {
            child.unmount(&native);
        }
    }

    pub fn replace(&self, old: &Self, new: &Self) {
        if let Some(parent) = new.parent() {
            parent.remove(new);
        }

        old.set_main_tree(false);
        new.set_main_tree(self.main_tree());

        {
            let mut children = self.0.children.borrow_mut();
            let index = children
                .iter()
                .position(|node| node == old)
                .expect("replacement target not a child of self");

            new.0.parent.replace(old.0.parent.take());

            new.0.next.replace(old.0.next.take());

            if let Some(prev) = index.checked_sub(1).and_then(|i| children.get(i)) {
                prev.0.next.replace(Some(new.clone()));
            }

            children[index] = new.clone();
        }

        if let Some(native) = self.to_native_anchor() {
            old.unmount(&native);
            let before = new.next_native_sibling();
            new.mount(&native, before.as_ref());
        }
    }

    // Text //

    pub fn set_text(&self, value: String) {
        if let NodeKind::Text(content) = &self.0.kind {
            self.native_ref().unwrap().set_text_content(Some(&value));
            *content.borrow_mut() = value;
        } else {
            self.expect_kind(Expect::Text);
        }
    }

    // Raw //

    pub fn set_raw_content(&self, value: String) {
        if let NodeKind::Raw(content) = &self.0.kind {
            for child in self.children() {
                self.remove(&child);
            }

            if is_browser() {
                let range = web_sys::Range::new().unwrap();
                let doc = range.create_contextual_fragment(&value).unwrap();
                let list = doc.child_nodes();

                for i in 0..list.length() {
                    let native = list.item(i).unwrap();
                    let holder = Self::fragment();
                    holder.manual_bind(native);
                    self.insert(&holder, None);
                }
            }

            *content.borrow_mut() = value;
        } else {
            self.expect_kind(Expect::Raw);
        }
    }

    // Elements //

    pub fn attr(&self, key: &str) -> Option<String> {
        if let NodeKind::Element { attrs, .. } = &self.0.kind {
            attrs.borrow().get(key).cloned()
        } else {
            self.expect_kind(Expect::Element);
        }
    }

    pub fn set_attr(&self, key: String, value: String) -> Option<String> {
        if let NodeKind::Element { attrs, .. } = &self.0.kind {
            if let Some(native) = self.native_ref() {
                intern(&key);
                native
                    .unchecked_ref::<web_sys::Element>()
                    .set_attribute(&key, &value)
                    .unwrap();
            }

            attrs.borrow_mut().insert(key, value)
        } else {
            self.expect_kind(Expect::Element);
        }
    }

    pub fn remove_attr(&self, key: &str) -> Option<String> {
        if let NodeKind::Element { attrs, .. } = &self.0.kind {
            if let Some(native) = self.native_ref() {
                native
                    .unchecked_ref::<web_sys::Element>()
                    .remove_attribute(key)
                    .unwrap();
            }

            attrs.borrow_mut().shift_remove(key)
        } else {
            self.expect_kind(Expect::Element);
        }
    }

    pub fn event<K, F>(&self, key: &K, options: EventOptions, mut f: F)
    where
        K: EventKey,
        F: FnMut(K::Event) + 'static,
    {
        if !matches!(self.0.kind, NodeKind::Element { .. }) {
            self.expect_kind(Expect::Element);
        }

        thread_local!(static ID: Cell<u64> = const { Cell::new(0) });
        let id = ID.replace(ID.get() + 1);

        let name = key.name();
        let opts = options.to_native(name);

        let node = self.clone();
        let closure = EventClosure::new(move |ev: web_sys::Event| {
            f(ev.dyn_into().expect("event type mismatch"));

            if options.once {
                node.browser().events.borrow_mut().remove(&id);
            }
        });

        if is_browser() {
            self.native_ref()
                .unwrap()
                .unchecked_ref::<web_sys::Element>()
                .add_event_listener_with_callback_and_add_event_listener_options(
                    intern(name),
                    closure.as_ref().unchecked_ref(),
                    &opts,
                )
                .unwrap();
        }

        self.browser().events.borrow_mut().insert(
            id,
            EventEntry {
                name: name.to_string(),
                closure,
                options,
            },
        );
    }

    // Other Utilities //

    pub fn element_name(&self) -> &str {
        if let NodeKind::Element { name, .. } = &self.0.kind {
            name
        } else {
            self.expect_kind(Expect::Element);
        }
    }

    pub fn element_namespace(&self) -> Option<&str> {
        if let NodeKind::Element { namespace, .. } = &self.0.kind {
            namespace.as_deref()
        } else {
            self.expect_kind(Expect::Element);
        }
    }

    fn parent(&self) -> Option<Self> {
        self.0.parent.borrow().as_ref().and_then(Self::upgrade)
    }

    pub fn children(&self) -> Vec<Self> {
        self.0.children.borrow().clone()
    }

    pub fn children_ref(&self) -> Ref<Vec<Self>> {
        self.0.children.borrow()
    }

    pub fn native(&self) -> Option<web_sys::Node> {
        self.native_ref().map(|native| native.clone())
    }

    pub fn native_ref(&self) -> Option<Ref<web_sys::Node>> {
        self.0
            .browser
            .as_ref()
            .and_then(|browser| Ref::filter_map(browser.native.borrow(), |n| n.as_ref()).ok())
    }

    // Internal //

    fn kind(&self) -> &NodeKind {
        &self.0.kind
    }

    fn browser(&self) -> &BrowserNode {
        self.0
            .browser
            .as_ref()
            .expect("not running within a browser environment")
    }

    fn manual_bind<N: Into<web_sys::Node>>(&self, node: N) {
        self.browser().native.replace(Some(node.into()));
    }

    fn main_tree(&self) -> bool {
        self.0.main_tree.get()
    }

    fn set_main_tree(&self, value: bool) {
        self.0.main_tree.set(value);

        for child in &*self.0.children.borrow() {
            child.set_main_tree(value);
        }

        match &self.0.kind {
            NodeKind::Component(component) if value => {
                component.on_mount();
            }
            _ => {}
        }
    }

    fn mount(&self, parent: &web_sys::Node, before: Option<&web_sys::Node>) {
        if let Some(native) = self.native_ref() {
            parent.insert_before(&native, before).unwrap();
        } else {
            for child in &*self.0.children.borrow() {
                child.mount(parent, before);
            }
        }
    }

    fn unmount(&self, parent: &web_sys::Node) {
        if let Some(native) = self.native_ref() {
            parent.remove_child(&native).unwrap();
        } else {
            for child in &*self.0.children.borrow() {
                child.unmount(parent);
            }
        }
    }

    /// Finds the nearest ancestral native node, **including** `self`.
    ///
    /// [`Node::insert`] uses the returned node as the mount point.
    fn to_native_anchor(&self) -> Option<web_sys::Node> {
        self.native()
            .or_else(|| self.parent().as_ref().and_then(Self::to_native_anchor))
    }

    /// Finds the nearest native sibling node, **excluding** `self`.
    ///
    /// The node returned is the next native sibling relative to `self`'s conceptual position within the native DOM tree.
    ///
    /// Consider the following node tree:
    /// ```
    /// div! {
    ///     fragment! {
    ///         fragment!(); // A
    ///         fragment!();
    ///     }
    ///     div!(); // B
    /// }
    /// ```
    /// Calling this method on `A` would return `B`, since it's the next native node under its native parent.
    /// If `B` were not there, it would return `None`, as there would be no subsequent native nodes.
    fn next_native_sibling(&self) -> Option<web_sys::Node> {
        if let Some(next) = &*self.0.next.borrow() {
            next.native().or_else(|| next.next_native_sibling())
        } else {
            self.parent().and_then(|parent| {
                if matches!(parent.0.kind, NodeKind::Element { .. }) {
                    None
                } else {
                    parent.next_native_sibling()
                }
            })
        }
    }

    /// Finds the nearest native sibling node, **including** `self`.
    ///
    /// See [Self::next_native_sibling] for an explanation.
    fn to_native_sibling(&self) -> Option<web_sys::Node> {
        self.native().or_else(|| self.next_native_sibling())
    }

    fn downgrade(&self) -> Weak<RawNode> {
        Rc::downgrade(&self.0)
    }

    fn upgrade(weak: &Weak<RawNode>) -> Option<Self> {
        weak.upgrade().map(Node)
    }

    fn expect_kind(&self, kind: Expect) -> ! {
        let expected = match kind {
            Expect::Container => "container",
            Expect::Element => "element",
            Expect::Text => "text",
            Expect::Raw => "raw",
        };
        let found = match &self.0.kind {
            NodeKind::Element { .. } => "element",
            NodeKind::Text(_) => "text",
            NodeKind::Raw(_) => "raw",
            NodeKind::Fragment => "fragment",
            NodeKind::Component(_) => "component",
        };

        panic!("expected {}, found {}", expected, found);
    }
}

impl Eq for Node {}
impl PartialEq for Node {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }
}

pub trait IntoNode: Sized {
    fn into_node(self) -> Node;

    fn replace_self(self, parent: &Node, target: &Node) -> Node {
        let new = self.into_node();
        parent.replace(target, &new);
        new
    }
}

impl IntoNode for Node {
    fn into_node(self) -> Self {
        self
    }
}

impl IntoNode for &str {
    fn into_node(self) -> Node {
        Node::text(self.to_string())
    }

    fn replace_self(self, parent: &Node, target: &Node) -> Node {
        <String as IntoNode>::replace_self(self.to_string(), parent, target)
    }
}

impl IntoNode for String {
    fn into_node(self) -> Node {
        Node::text(self)
    }

    fn replace_self(self, parent: &Node, target: &Node) -> Node {
        match target.0.kind {
            NodeKind::Text(_) => {
                target.set_text(self);
                target.clone()
            }
            NodeKind::Raw(_) => {
                target.set_raw_content(self);
                target.clone()
            }
            _ => self.into_node().replace_self(parent, target),
        }
    }
}

impl<N> FromIterator<N> for Node
where
    N: IntoNode,
{
    fn from_iter<T: IntoIterator<Item = N>>(iter: T) -> Self {
        iter.into_iter().fold(Self::fragment(), |fragment, node| {
            fragment.insert(&node.into_node(), None);
            fragment
        })
    }
}

enum Expect {
    Container,

    Element,
    Text,
    Raw,
}
