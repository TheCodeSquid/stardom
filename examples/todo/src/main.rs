use stardom::prelude::*;

fn main() {
    console_error_panic_hook::set_once();

    stardom::mount(app, "#root");
}

#[component]
fn app() -> Node {
    let count = signal(0i32);
    let increment = move |_| {
        count.update(|n| *n += 1);
    };
    let decrement = move |_| {
        count.update(|n| *n -= 1);
    };

    let state = memo(move || if count.get() % 2 == 0 { "Even" } else { "Odd" });

    div! {
        h1!("Hello, world!");

        p! {
            {count.get().to_string()};
            " is ";
            {state.get()};
        };

        button! {
            @click => increment;
            "Increment";
        };
        button! {
            @click => decrement;
            "Decrement";
        }
    }
}
