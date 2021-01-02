#![feature(custom_test_frameworks)]
#![test_runner(iai::runner)]

use iai::black_box;
use iai::iai;

fn fibonacci(n: u64) -> u64 {
    match n {
        0 | 1 => 1,
        n => fibonacci(n - 1) + fibonacci(n - 2),
    }
}

#[iai]
fn bench_empty() {
    return;
}

#[iai]
fn bench_fibonacci() -> u64 {
    fibonacci(black_box(10))
}

#[iai]
fn bench_fibonacci_long() -> u64 {
    fibonacci(black_box(30))
}
