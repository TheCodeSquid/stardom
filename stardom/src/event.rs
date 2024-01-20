use std::borrow::Cow;

use wasm_bindgen::JsCast;

#[derive(Clone, Copy, PartialEq, Eq, Default, Debug)]
pub struct EventOptions {
    pub capture: Option<bool>,
    pub once: Option<bool>,
    pub passive: Option<bool>,
}

impl EventOptions {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn capture(mut self, value: bool) -> Self {
        self.capture = Some(value);
        self
    }

    pub fn once(mut self, value: bool) -> Self {
        self.once = Some(value);
        self
    }

    pub fn passive(mut self, value: bool) -> Self {
        self.passive = Some(value);
        self
    }
}

impl From<EventOptions> for web_sys::AddEventListenerOptions {
    fn from(value: EventOptions) -> Self {
        let mut opts = Self::new();
        if let Some(capture) = value.capture {
            opts.capture(capture);
        }
        if let Some(once) = value.once {
            opts.capture(once);
        }
        if let Some(passive) = value.passive {
            opts.passive(passive);
        }
        opts
    }
}

pub trait EventKey {
    type Event: JsCast;

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
        self
    }
}

impl<'a> EventKey for Cow<'a, str> {
    type Event = web_sys::Event;

    fn name(&self) -> &str {
        self
    }
}
