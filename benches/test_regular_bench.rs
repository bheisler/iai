use iai::{black_box, Iai};

fn fibonacci(n: u64) -> u64 {
    match n {
        0 | 1 => 1,
        n => fibonacci(n - 1) + fibonacci(n - 2),
    }
}

fn bench_empty(iai: &mut Iai) {
    iai.run(|| {
        return;
    });
}

fn bench_fibonacci(iai: &mut Iai) {
    iai.run(|| fibonacci(black_box(10)));
}

fn bench_fibonacci_long(iai: &mut Iai) {
    let target = black_box(2_u64.pow(4) + 7 * 2); // 30
    iai.run(|| fibonacci(black_box(target)));
}

iai::main!(bench_empty, bench_fibonacci, bench_fibonacci_long);
