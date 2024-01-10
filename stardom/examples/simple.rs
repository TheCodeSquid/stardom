use stardom::*;

fn main() {}

#[component]
fn app() -> Node {
    div! {
        class => "app";

        h1!("Hello, world!");
    }
}
