use stardom::*;

fn main() {
    console_error_panic_hook::set_once();

    stardom::mount(app(), "#root");
}

#[component]
fn app() -> Node {
    div! {
        class => "app";

        h1! { "Hello, ";em!("stardom");"!" };
    }
}
