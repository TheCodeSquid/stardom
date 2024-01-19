use std::{
    cell::{Cell, RefCell},
    collections::{HashMap, HashSet},
    rc::Rc,
    thread_local,
};

use super::{effect::Effect, handle::Handle, signal::RawSignal};

thread_local! {
    pub(crate) static RUNTIME: RefCell<Option<Runtime>> = const { RefCell::new(None) };
}

pub(crate) struct Runtime {
    pub(super) handle_cycle: u64,
    pub(super) scopes: RefCell<HashMap<Handle, HashSet<Handle>>>,
    pub(super) signals: RefCell<HashMap<Handle, RawSignal>>,

    pub(super) current_scope: Cell<Handle>,
    pub(super) current_effect: RefCell<Option<Rc<Effect>>>,
}

impl Default for Runtime {
    fn default() -> Self {
        let cycle = Handle::next_cycle();

        Self {
            handle_cycle: cycle,
            scopes: RefCell::default(),
            signals: RefCell::default(),
            current_scope: Cell::new(Handle::new(cycle)),
            current_effect: RefCell::default(),
        }
    }
}
