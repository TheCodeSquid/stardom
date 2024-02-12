#![warn(clippy::use_self)]

extern crate self as stardom_core;

#[path = "macro.rs"]
pub mod __macro;

pub mod component;

pub mod attrs;
pub mod events;
pub mod util;

mod env;
mod node;
mod node_ref;

pub use node::*;
pub use node_ref::*;

pub mod named {
    pub mod elements {
        stardom_macros::create_named!(elements);
    }

    pub mod attrs {
        stardom_macros::create_named!(attributes);
    }

    pub mod events {
        stardom_macros::create_named!(events);
    }
}

pub mod bindings {
    pub use stardom_macros::{bind_this as this, bind_value as value};
}
