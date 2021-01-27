# `criterion-macro`

This crate provides a procedural macro that allows the use of `#[iai]` to mark [Iai]
benchmark functions.

## License

This project is licensed under either of

* [Apache License, Version 2.0](http://www.apache.org/licenses/LICENSE-2.0)
  ([LICENSE-APACHE](LICENSE-APACHE))

* [MIT License](http://opensource.org/licenses/MIT)
  ([LICENSE-MIT](LICENSE-MIT))

at your option.

## Example

To use the this, create a benchmark `benches/bench_with_macro.rs` with this content:
```rust
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
```

Then add this in your `Cargo.toml`:
```
[dev-dependencies]
iai = { version = "0.1.1", default-features = false, features = ["macro"] }

[[bench]]
name = "bench_with_macro"
```

Note that you should not disable the testing harness when using the macro.

Now run the benchmark with `cargo bench`


## Contributing

We welcome all people who want to contribute.
Please see the contributing instructions in the base repository for more information.

Contributions in any form (issues, pull requests, etc.) to this project
must adhere to Rust's [Code of Conduct].

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in this crate by you, as defined in the Apache-2.0 license, shall
be dual licensed as above, without any additional terms or conditions.

[Code of Conduct]: https://www.rust-lang.org/en-US/conduct.html
[Iai]: https://github.com/bheisler/iai
