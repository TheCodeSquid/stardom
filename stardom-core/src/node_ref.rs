use std::{cell::RefCell, rc::Rc};

use crate::node::Node;

#[derive(Clone, Default)]
pub struct NodeRef(Rc<RefCell<Option<Node>>>);

impl NodeRef {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn try_get(&self) -> Option<Node> {
        self.0.borrow().clone()
    }

    pub fn get(&self) -> Node {
        self.try_get().expect("NodeRef accessed before assignment")
    }

    pub fn set(&self, node: Node) -> Option<Node> {
        self.0.replace(Some(node))
    }
}

impl Eq for NodeRef {}
impl PartialEq for NodeRef {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }
}
