use std::{
    cell::{Ref, RefCell, RefMut},
    fmt::{self, Write},
    rc::{Rc, Weak},
};

use indexmap::IndexMap;
use stardom_nodes::{EventKey, Node};

// Reference: https://developer.mozilla.org/en-US/docs/Glossary/Void_element
const VOID_ELEMENTS: &[&str] = &[
    "area", "base", "br", "col", "embed", "hr", "img", "input", "link", "meta", "param", "source",
    "track", "wbr",
];

#[derive(Clone, Debug)]
pub struct NodeRef(Rc<RefCell<Inner>>);

type WeakNode = Weak<RefCell<Inner>>;

struct Inner {
    parent: Option<WeakNode>,
    kind: NodeKind,
}

#[derive(Debug)]
pub enum NodeKind {
    Element {
        namespace: Option<String>,
        name: String,
        attrs: IndexMap<String, String>,
        children: Vec<NodeRef>,
    },
    Fragment(Vec<NodeRef>),
    Text(String),
    Raw(String),
}

impl fmt::Display for NodeRef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let inner = self.0.borrow();
        match &inner.kind {
            NodeKind::Element {
                namespace,
                name,
                attrs,
                children,
            } => {
                let tag = namespace
                    .as_ref()
                    .map(|ns| format!("{ns}:{name}"))
                    .unwrap_or(name.clone());

                let mut buf = String::new();
                for child in children {
                    writeln!(&mut buf, "{child}")?;
                }
                let indented = buf
                    .lines()
                    .map(|line| format!("  {line}"))
                    .collect::<Vec<_>>()
                    .join("\n");

                let attrs = attrs
                    .iter()
                    .map(|(name, value)| format!(" {name}=\"{}\"", escape(value)))
                    .collect::<Vec<_>>()
                    .join("");

                if !indented.is_empty() {
                    write!(f, "<{tag}{attrs}>\n{indented}\n</{tag}>")
                } else if VOID_ELEMENTS.contains(&tag.to_lowercase().as_str()) {
                    write!(f, "<{tag}{attrs}>")
                } else {
                    write!(f, "<{tag}{attrs}></{tag}>")
                }
            }
            NodeKind::Fragment(children) => {
                for child in children {
                    writeln!(f, "{child}")?;
                }
                Ok(())
            }
            NodeKind::Text(text) => {
                write!(f, "{}", escape(text))
            }
            NodeKind::Raw(raw) => {
                write!(f, "{raw}")
            }
        }
    }
}

impl NodeRef {
    fn new(kind: NodeKind) -> Self {
        let inner = Inner { parent: None, kind };
        Self(Rc::new(RefCell::new(inner)))
    }

    fn children_ref(&self) -> Option<Ref<Vec<Self>>> {
        let inner = self.0.borrow();
        Ref::filter_map(inner, |inner| match &inner.kind {
            NodeKind::Element { children, .. } => Some(children),
            NodeKind::Fragment(children) => Some(children),
            _ => None,
        })
        .ok()
    }

    fn children_mut(&self) -> Option<RefMut<Vec<Self>>> {
        let inner = self.0.borrow_mut();
        RefMut::filter_map(inner, |inner| match &mut inner.kind {
            NodeKind::Element { children, .. } => Some(children),
            NodeKind::Fragment(children) => Some(children),
            _ => None,
        })
        .ok()
    }
}

impl Node for NodeRef {
    fn element(namespace: Option<&str>, name: &str) -> Self {
        let kind = NodeKind::Element {
            namespace: namespace.map(str::to_string),
            name: name.to_string(),
            attrs: IndexMap::new(),
            children: vec![],
        };
        Self::new(kind)
    }

    fn text() -> Self {
        let kind = NodeKind::Text(String::new());
        Self::new(kind)
    }

    fn fragment() -> Self {
        let kind = NodeKind::Fragment(vec![]);
        Self::new(kind)
    }

    fn raw() -> Self {
        let kind = NodeKind::Raw(String::new());
        Self::new(kind)
    }

    fn parent(&self) -> Option<Self> {
        self.0
            .borrow()
            .parent
            .as_ref()
            .and_then(Weak::upgrade)
            .map(NodeRef)
    }

    fn children(&self) -> Vec<Self> {
        self.children_ref()
            .expect("only element and fragment nodes can have children")
            .clone()
    }

    fn next_sibling(&self) -> Option<Self> {
        let parent = self.parent()?;
        let children = parent.children_ref()?;

        let idx = children.iter().position(|node| node == self)?;
        children.get(idx + 1).cloned()
    }

    fn insert(&self, child: &Self, before: Option<&Self>) {
        let mut children = self
            .children_mut()
            .expect("only element and fragment nodes can have children");
        let idx = if let Some(before) = before {
            children
                .iter()
                .position(|node| node == before)
                .expect("not a parent of insertion point node")
        } else {
            children.len()
        };
        children.insert(idx, child.clone());

        child.0.borrow_mut().parent.replace(Rc::downgrade(&self.0));
    }

    fn remove(&self, child: &Self) {
        let mut children = self
            .children_mut()
            .expect("only element and fragment nodes can have children");
        let idx = children
            .iter()
            .position(|node| node == child)
            .expect("not a parent of child node");
        children.remove(idx);

        child.0.borrow_mut().parent.take();
    }

    fn set_text(&self, content: &str) {
        match &mut self.0.borrow_mut().kind {
            NodeKind::Text(text) => {
                *text = content.to_string();
            }
            NodeKind::Raw(raw) => {
                *raw = content.to_string();
            }
            _ => panic!("can only set text content of text or raw nodes"),
        }
    }

    fn set_attr(&self, name: &str, value: &str) {
        if let NodeKind::Element { attrs, .. } = &mut self.0.borrow_mut().kind {
            attrs.insert(name.to_string(), value.to_string());
        } else {
            panic!("attributes only exist on element nodes");
        }
    }

    fn remove_attr(&self, name: &str) {
        if let NodeKind::Element { attrs, .. } = &mut self.0.borrow_mut().kind {
            attrs.remove(name);
        } else {
            panic!("attributes only exist on element nodes");
        }
    }

    fn event<E, F>(&self, _event: &E, _f: F)
    where
        E: EventKey,
        F: Fn(E::Event) + 'static,
    {
        // no-op, since events don't exist on renders
    }
}

impl PartialEq for NodeRef {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }
}

impl Eq for NodeRef {}

impl fmt::Debug for Inner {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.kind.fmt(f)
    }
}

// Reference: https://cheatsheetseries.owasp.org/cheatsheets/Cross_Site_Scripting_Prevention_Cheat_Sheet.html#output-encoding-for-html-contexts
fn escape(text: &str) -> String {
    let mut output = String::new();
    for c in text.chars() {
        match c {
            '&' => output.push_str("&amp;"),
            '<' => output.push_str("&lt;"),
            '>' => output.push_str("&gt"),
            '"' => output.push_str("&quot;"),
            '\'' => output.push_str("&#x27"),
            _ => output.push(c),
        }
    }
    output
}

#[cfg(test)]
mod tests {
    use super::NodeRef;

    #[test]
    fn node_behavior() {
        stardom_nodes::behavior_tests!(NodeRef);
    }
}
