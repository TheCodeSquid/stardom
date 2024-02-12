use std::borrow::Cow;

use crate::node::Node;

pub trait IntoAttr<'a>: Sized {
    fn into_attr(self) -> Option<Cow<'a, str>>;

    fn set_attr(self, node: &Node, key: Cow<str>) {
        if let Some(value) = self.into_attr() {
            node.set_attr(key.into_owned(), value.into_owned());
        } else {
            node.remove_attr(key.as_ref());
        }
    }
}

impl<'a> IntoAttr<'a> for &'a str {
    fn into_attr(self) -> Option<Cow<'a, str>> {
        Some(Cow::Borrowed(self))
    }
}

impl<'a> IntoAttr<'a> for Cow<'a, str> {
    fn into_attr(self) -> Option<Cow<'a, str>> {
        Some(self)
    }
}

impl<'a> IntoAttr<'a> for String {
    fn into_attr(self) -> Option<Cow<'a, str>> {
        Some(Cow::Owned(self))
    }
}

impl<'a> IntoAttr<'a> for &'a String {
    fn into_attr(self) -> Option<Cow<'a, str>> {
        Some(Cow::Borrowed(self))
    }
}

impl<'a> IntoAttr<'a> for Option<&'a str> {
    fn into_attr(self) -> Option<Cow<'a, str>> {
        self.map(Cow::Borrowed)
    }
}

impl<'a> IntoAttr<'a> for Option<Cow<'a, str>> {
    fn into_attr(self) -> Self {
        self
    }
}

impl<'a> IntoAttr<'a> for Option<String> {
    fn into_attr(self) -> Option<Cow<'a, str>> {
        self.map(Cow::Owned)
    }
}

impl<'a> IntoAttr<'a> for Option<&'a String> {
    fn into_attr(self) -> Option<Cow<'a, str>> {
        self.map(|s| Cow::Borrowed(s.as_str()))
    }
}
