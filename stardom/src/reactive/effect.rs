use std::rc::Rc;

use super::item::Item;

#[derive(Clone)]
pub(crate) struct Effect {
    pub item: Rc<Item>,
}

impl Effect {
    pub fn new<F>(f: F) -> Self
    where
        F: Fn() + 'static,
    {
        let item = Rc::new(Item {
            action: Some(Box::new(f)),
            ..Item::new()
        });
        item.run();
        Self { item }
    }
}
