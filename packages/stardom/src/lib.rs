pub use stardom_nodes as nodes;
pub use stardom_reactive as reactive;

#[cfg(feature = "web")]
pub use stardom_web as web;

#[cfg(feature = "render")]
pub use stardom_render as render;

#[cfg(feature = "web")]
pub use web::mount;

pub mod prelude {
    #[doc(hidden)]
    pub use stardom_macros;
    #[doc(hidden)]
    pub use stardom_nodes;
    #[doc(hidden)]
    pub use stardom_reactive;

    pub use crate::{
        nodes::*,
        reactive::{effect, lazy_effect, memo, signal, untrack, Track, Trigger},
    };
}

pub fn init() {
    reactive::Runtime::new().init();
}
