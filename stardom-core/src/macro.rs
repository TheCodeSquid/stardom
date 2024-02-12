pub use web_sys;

pub mod bindings {
    use stardom_reactive::Output;
    use wasm_bindgen::JsCast;

    use crate::{events::EventOptions, node::Node};

    pub fn bind_value<O>(node: &Node, output: O)
    where
        O: Output<String> + 'static,
    {
        node.event(&"input", EventOptions::new(), move |ev| {
            let value = ev
                .current_target()
                .unwrap()
                .unchecked_into::<web_sys::HtmlInputElement>()
                .value();
            output.set(value);
        });
    }
}
