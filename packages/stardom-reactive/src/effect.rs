use crate::{
    item::{Item, ItemKey},
    runtime::Runtime,
};

#[derive(Clone, Copy)]
pub struct Effect {
    rt: &'static Runtime,
    item: ItemKey,
}

impl Effect {
    pub(crate) fn new<F: Fn() + 'static>(rt: &'static Runtime, f: F) -> Self {
        let item = rt.register(Item {
            action: Some(Box::new(f)),
            ..Default::default()
        });
        Self { rt, item }
    }

    pub fn run(&self) {
        self.item.run(self.rt)
    }
}
