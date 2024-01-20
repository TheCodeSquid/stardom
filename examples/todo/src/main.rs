use stardom::prelude::*;

fn main() {
    console_error_panic_hook::set_once();

    stardom::mount(app, "#root");
}

#[component]
fn app() -> Node {
    let count = signal(0i64);
    let increment = move |_ev| count.update(|n| *n += 1);
    let decrement = move |_ev| count.update(|n| *n -= 1);

    let hi = memo(move || {
        let n = count.get();
        if n >= 0 {
            (0..n)
                .map(|i| div!("Hi :3 — #", i.to_string()))
                .collect::<Vec<_>>()
        } else {
            (0..-n).map(|i| div!("Bye :3 — #", i.to_string())).collect()
        }
    });

    div! {
        class => "app";

        h1!("Hello, ", em!("stardom"), "!");
        p!("Count: ", {count.get().to_string()});

        button! {
            @click => increment;
            "Increment"
        }
        button! {
            @click => decrement;
            "Decrement"
        }

        hr!();

        thing() {
            // uncomment for an error
            // class => "thing";

            {hi.cloned()}
        }
    }
}

fn thing(children: Node) -> Node {
    div! {
        em!(style => "display: block", "children start");
        children;
        em!(style => "display: block", "children end");
    }
}
