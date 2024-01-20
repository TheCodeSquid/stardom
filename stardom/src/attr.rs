use std::borrow::Cow;

use crate::node::Node;

pub struct Attr(pub Option<String>);

impl Attr {
    pub fn apply(self, node: &Node, key: impl Into<String>) {
        if let Some(value) = self.0 {
            node.set_attr(key, value);
        } else {
            node.remove_attr(key);
        }
    }
}

impl From<&str> for Attr {
    fn from(value: &str) -> Self {
        Self(Some(value.to_owned()))
    }
}

impl From<String> for Attr {
    fn from(value: String) -> Self {
        Self(Some(value))
    }
}

impl<'a> From<Cow<'a, str>> for Attr {
    fn from(value: Cow<'a, str>) -> Self {
        Self(Some(value.into_owned()))
    }
}

impl From<Option<&str>> for Attr {
    fn from(value: Option<&str>) -> Self {
        Self(value.map(ToOwned::to_owned))
    }
}

impl From<Option<String>> for Attr {
    fn from(value: Option<String>) -> Self {
        Self(value)
    }
}

impl<'a> From<Option<Cow<'a, str>>> for Attr {
    fn from(value: Option<Cow<'a, str>>) -> Self {
        Self(value.map(Cow::into_owned))
    }
}
