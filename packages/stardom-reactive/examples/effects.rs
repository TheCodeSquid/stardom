use stardom_reactive::{effect, signal, Read, Run, Write};

fn main() {
    stardom_reactive::init();

    let count = signal(0);

    let printer = effect(move || {
        println!("count: {}", count.get());
    });

    count.set(1);
    count.update(|n| *n += 1);

    printer.run();
}
