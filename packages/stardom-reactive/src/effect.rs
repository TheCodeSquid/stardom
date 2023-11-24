use crate::{
    item::{Item, Run},
    ItemKey, Runtime,
};

#[derive(Clone, Copy)]
pub struct Effect {
    rt: &'static Runtime,
    key: ItemKey,
}

impl Effect {
    pub fn new<F>(rt: &'static Runtime, parent: Option<ItemKey>, f: F) -> Self
    where
        F: FnMut() + 'static,
    {
        let item = Item {
            action: Some(Box::new(f)),
            parent,
            ..Default::default()
        };

        Self {
            rt,
            key: rt.add(item),
        }
    }
}

impl Run for Effect {
    fn run(&self) {
        self.key.run(self.rt);
    }
}
