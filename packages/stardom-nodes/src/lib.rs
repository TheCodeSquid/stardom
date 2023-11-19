mod test;

mod macros;

use wasm_bindgen::{convert::FromWasmAbi, JsCast};

pub trait Node: Clone + Sized + 'static {
    fn element(namespace: Option<&str>, name: &str) -> Self;

    fn text() -> Self;

    fn fragment() -> Self;

    fn raw() -> Self;

    fn parent(&self) -> Option<Self>;

    fn children(&self) -> Vec<Self>;

    fn next_sibling(&self) -> Option<Self>;

    fn insert(&self, child: &Self, before: Option<&Self>);

    fn remove(&self, child: &Self);

    fn set_text(&self, content: &str);

    fn set_attr(&self, name: &str, value: &str);

    fn remove_attr(&self, name: &str);

    fn event<E, F>(&self, event: &E, f: F)
    where
        E: EventKey,
        F: Fn(E::Event) + 'static;
}

pub trait EventKey {
    type Event: FromWasmAbi + JsCast;

    fn name(&self) -> &str;
}

impl EventKey for &str {
    type Event = web_sys::Event;

    fn name(&self) -> &str {
        self
    }
}

impl EventKey for String {
    type Event = web_sys::Event;

    fn name(&self) -> &str {
        self.as_str()
    }
}
