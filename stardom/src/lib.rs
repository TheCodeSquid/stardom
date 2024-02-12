extern crate self as stardom;

// Re-exports //

pub use stardom_core::{
    self as core,
    browser::mount,
    hydrate::hydrate,
    render::{render, render_to_string},
    util::{document, window},
    IntoNode, Node,
};
pub use stardom_reactive as reactive;

pub mod prelude {
    pub use stardom_core::{
        component::{on_mount, on_unmount},
        named::elements::*,
        Node, NodeRef,
    };
    pub use stardom_macros::{component, element, fragment};
    pub use stardom_reactive::{
        batch, effect, lazy_effect, memo, signal, untrack, Input as _, Output as _, Track as _,
        Trigger as _,
    };

    // Hidden for macros
    #[doc(hidden)]
    pub use stardom_core;
    #[doc(hidden)]
    pub use stardom_reactive;
}
