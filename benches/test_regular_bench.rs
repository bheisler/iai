use iai::black_box;

fn fibonacci(n: u64) -> u64 {
    match n {
        0 | 1 => 1,
        n => fibonacci(n - 1) + fibonacci(n - 2),
    }
}

fn bench_empty() {
    return;
}

fn bench_fibonacci() -> u64 {
    fibonacci(black_box(10))
}

fn bench_fibonacci_long() -> u64 {
    fibonacci(black_box(30))
}

iai::main!(bench_empty, bench_fibonacci, bench_fibonacci_long);
