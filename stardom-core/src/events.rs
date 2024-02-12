use wasm_bindgen::JsCast;

#[derive(Clone, Copy, Default, Debug)]
pub struct EventOptions {
    pub capture: bool,
    pub once: bool,
    pub passive: Option<bool>,
}

impl EventOptions {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn capture(mut self, value: bool) -> Self {
        self.capture = value;
        self
    }

    pub fn once(mut self, value: bool) -> Self {
        self.once = value;
        self
    }

    pub fn passive(mut self, value: bool) -> Self {
        self.passive = Some(value);
        self
    }

    pub fn to_native(self, event: &str) -> web_sys::AddEventListenerOptions {
        let Self {
            capture,
            once,
            passive,
        } = self;

        let passive = match passive {
            Some(value) => value,
            // TODO: check if any others should be passive by default
            None => matches!(event, "scroll" | "wheel"),
        };

        let mut opts = web_sys::AddEventListenerOptions::new();
        opts.capture(capture).once(once).passive(passive);
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
