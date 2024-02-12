use stardom::{document, prelude::*};

fn main() {
    let root = document().query_selector("#root").unwrap().unwrap();

    stardom::mount(root, app);
}

#[component]
fn app() -> Node {
    div! { class => "app";
        h1!("Hello, Stardom!");
    }
}
