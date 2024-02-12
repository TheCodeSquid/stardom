use std::mem;

use wasm_bindgen::prelude::*;

use crate::{
    env::{self, Env},
    node::{Node, NodeKind},
    util::document,
};

pub fn hydrate<F>(root: web_sys::Node, f: F)
where
    F: FnOnce() -> Node,
{
    env::replace(Env::Hydrate);

    let root_node = Node::fragment();
    let provided = f();
    root_node.insert(&provided, None);

    let mut hydrator = Hydrator::new(root.clone());
    hydrator.hydrate(&provided);

    env::replace(Env::Browser);

    root_node.manual_bind(root);
    root_node.set_main_tree(true);
    mem::forget(root_node);
}

const MARKER: &str = "stardom:raw";

#[derive(Clone, Copy)]
enum Step {
    Next,
    Over,
}
use Step::*;

struct Hydrator {
    tree: web_sys::TreeWalker,
    parent: web_sys::Node,
}

impl Hydrator {
    fn new(root: web_sys::Node) -> Self {
        Self {
            tree: document().create_tree_walker(&root).unwrap(),
            parent: root,
        }
    }

    fn hydrate(&mut self, node: &Node) {
        match node.kind() {
            NodeKind::Element { name, .. } => {
                self.skip_whitespace();
                let native = self.step_element();
                let prev = mem::replace(&mut self.parent, native.clone().into());

                for event in node.browser().events.borrow().values() {
                    native
                        .add_event_listener_with_callback_and_add_event_listener_options(
                            &event.name,
                            event.closure.as_ref().unchecked_ref(),
                            &event.options.to_native(name),
                        )
                        .unwrap();
                }

                for child in &*node.children_ref() {
                    self.hydrate(child);
                }
                node.manual_bind(native);
                self.parent = prev;
            }
            NodeKind::Text(content) => {
                let native = self.step_text();
                native.set_text_content(Some(&content.borrow()));
                node.manual_bind(native);
            }
            NodeKind::Raw(_) => {
                self.skip_whitespace();
                self.step_marker();

                loop {
                    if is_marker(&self.peek(Over)) {
                        self.consume(Over);
                        break;
                    }
                    let native = self.step(Over);
                    let holder = Node::fragment();
                    node.insert(&holder, None);
                    holder.manual_bind(native);
                }
            }
            NodeKind::Fragment | NodeKind::Component(_) => {
                for child in &*node.children_ref() {
                    self.hydrate(child);
                }
            }
        }
    }

    fn try_step(&mut self, method: Step) -> Option<web_sys::Node> {
        match method {
            Step::Next => self.tree.next_node().unwrap(),
            Step::Over => self
                .tree
                .next_sibling()
                .unwrap()
                .or_else(|| self.tree.next_node().unwrap()),
        }
    }

    fn try_peek(&mut self, method: Step) -> Option<web_sys::Node> {
        let current = self.tree.current_node();
        let next = self.try_step(method);
        self.tree.set_current_node(&current);
        next
    }

    fn step(&mut self, method: Step) -> web_sys::Node {
        self.try_step(method).expect("hydrator reached section end")
    }

    fn peek(&mut self, method: Step) -> web_sys::Node {
        self.try_peek(method).expect("hydrator reached section end")
    }

    fn consume(&mut self, method: Step) -> web_sys::Node {
        let current = self.tree.current_node();
        let node = self.step(method);
        node.parent_node().unwrap().remove_child(&node).unwrap();
        self.tree.set_current_node(&current);
        node
    }

    fn skip_whitespace(&mut self) {
        while self.peek(Next).has_type::<web_sys::Text>() {
            self.consume(Next);
        }
    }

    fn step_element(&mut self) -> web_sys::Element {
        self.step(Next)
            .dyn_into()
            .unwrap_or_else(|node| mismatch("element", &node))
    }

    fn step_text(&mut self) -> web_sys::Text {
        if let Some(peeked) = self
            .try_peek(Step::Next)
            .and_then(|node| node.dyn_into::<web_sys::Text>().ok())
        {
            // guards for the condition where the next node is text, but exists outside of the current element:
            // div! {
            //   (current)
            // }
            // "B" << peeked
            if self.parent.contains(Some(&peeked)) {
                self.step(Next);
                return peeked;
            }
        }

        let current = self.tree.current_node();
        if !current.has_type::<web_sys::Text>() {
            mismatch("text", &current);
        }
        let text = web_sys::Text::new().unwrap();
        current
            .parent_node()
            .unwrap()
            .insert_before(&text, current.next_sibling().as_ref())
            .unwrap();
        self.tree.set_current_node(&text);
        text
    }

    fn step_marker(&mut self) {
        let node = self.consume(Next);
        if !is_marker(&node) {
            mismatch("raw section marker", &node);
        }
    }
}

fn is_marker(node: &web_sys::Node) -> bool {
    node.dyn_ref::<web_sys::Comment>()
        .map(|comment| comment.data() == MARKER)
        .unwrap_or(false)
}

fn mismatch(expected: &str, found: &web_sys::Node) -> ! {
    let name = js_sys::Reflect::get_prototype_of(found)
        .unwrap()
        .constructor()
        .name();
    panic!("hydrator expected {expected}, found {name}")
}
