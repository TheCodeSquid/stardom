use stardom_nodes::Node;

pub trait Dehydrate: Node {
    fn dehydrate(&self);
}

pub trait Hydrate: Node {
    type Native;

    fn hydrate(&self, native: Self::Native);
}
