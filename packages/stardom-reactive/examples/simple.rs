use stardom_reactive::{runtime::*, Track, Trigger};

fn main() {
    Runtime::new().init();

    let count = signal(0_u32);
    let double = count.map(|n| n * 2);
    let triple = signal(0_u32);

    effect(move || {
        println!("{:>2} * 2 = {}", untrack(|| count.get()), double.get());

        triple.set(untrack(|| count.get()) * 3);
    });

    effect(move || {
        println!("triple: {}", triple.get());
    });

    for i in 1..=10 {
        count.set(i);
    }
}
