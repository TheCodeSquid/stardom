use stardom::prelude::*;

#[component]
fn app() -> Node {
    let count = signal(0i32);

    let onclick = move |_| {
        count.update(|n| {
            *n += 1;
            gloo::console::log!("n: ", *n);
        });
    };

    div! {
        button! {
            @click => onclick;
            "Increment";
        };

        div! {
            match count.get() {
                0 => "zero";
                1 => "one";
                2 => "two";
                3 => "three";
                n => n.get().to_string();
            }
        }
    }
}

fn main() {
    console_error_panic_hook::set_once();

    stardom::mount(app, "#root");
}
