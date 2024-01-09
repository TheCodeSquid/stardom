use gloo::console;
use stardom::*;

fn main() {
    console_error_panic_hook::set_once();

    stardom::mount(app, "#root");
}

#[component]
fn app() -> Node {
    let count = signal(0);

    let text = Node::text("");
    effect!(count, text; {
        text.set_text(format!("count: {}", count.get()));
    });

    div! {
        class => "app";

        h1! { "Hello, ";em!("stardom");"!" };
        p! { text };

        button! {
            "Increment";
            @click => clone!(count; move |_ev| {
                count.update(|n| *n += 1);
            });
        }
    }
}
