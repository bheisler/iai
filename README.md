<h1 align="center">Iai</h1>

<div align="center">Experimental One-shot Benchmark Framework in Rust</div>

<div align="center">
    <img src="https://github.com/bheisler/iai/workflows/Continuous%20integration/badge.svg" alt="Continuous integration">
</div>

<div align="center">
	<a href="https://bheisler.github.io/criterion.rs/book/iai/getting_started.html">Getting Started</a>
    |
    <a href="https://bheisler.github.io/criterion.rs/book/iai/iai.html">User Guide</a>
    |
    <a href="https://docs.rs/crate/iai/">Released API Docs</a>
    |
    <a href="https://github.com/bheisler/iai/blob/master/CHANGELOG.md">Changelog</a>
</div>

Iai is an experimental benchmarking harness that uses Cachegrind to perform extremely precise
single-shot measurements of Rust code.

## Table of Contents
- [Table of Contents](#table-of-contents)
  - [Features](#features)
  - [Quickstart](#quickstart)
  - [Goals](#goals)
  - [Comparison with Criterion-rs](#comparison-with-criterion-rs)
  - [Contributing](#contributing)
  - [Compatibility Policy](#compatibility-policy)
  - [Maintenance](#maintenance)
  - [License](#license)

### Features

- __Precision__: High-precision measurements allow you to reliably detect very small optimizations to your code
- __Consistency__: Iai can take accurate measurements even in virtualized CI environments
- __Performance__: Since Iai only executes a benchmark once, it is typically faster to run than statistical benchmarks
- __Profiling__: Iai generates a Cachegrind profile of your code while benchmarking, so you can use Cachegrind-compatible tools to analyze the results in detail
- __Stable-compatible__: Benchmark your code without installing nightly Rust

### Quickstart

In order to use Iai, you must have [Valgrind] installed. This means that Iai cannot be used on
platforms that are not supported by Valgrind.

[Valgrind]: https://www.valgrind.org

To start with Iai, add the following to your `Cargo.toml` file:

```toml
[dev-dependencies]
iai = "0.1"

[[bench]]
name = "my_benchmark"
harness = false
```

Next, define a benchmark by creating a file at `$PROJECT/benches/my_benchmark.rs` with the following contents:

```rust
use iai::{black_box, main};

fn fibonacci(n: u64) -> u64 {
    match n {
        0 => 1,
        1 => 1,
        n => fibonacci(n-1) + fibonacci(n-2),
    }
}

fn iai_benchmark_short() -> u64 {
    fibonacci(black_box(10))
}

fn iai_benchmark_long() -> u64 {
    fibonacci(black_box(30));
}


iai::main!(iai_benchmark_short, iai_benchmark_long);
```

Finally, run this benchmark with `cargo bench`. You should see output similar to the following:

```
     Running target/release/deps/test_regular_bench-8b173c29ce041afa

bench_fibonacci_short
  Instructions:                1735
  L1 Accesses:                 2364
  L2 Accesses:                    1
  RAM Accesses:                   1
  Estimated Cycles:            2404

bench_fibonacci_long
  Instructions:            26214735
  L1 Accesses:             35638623
  L2 Accesses:                    2
  RAM Accesses:                   1
  Estimated Cycles:        35638668
```

### Goals

The primary goal of Iai is to provide a simple and precise tool for reliably detecting very small changes to the performance of code. Additionally, it should be as programmer-friendly as possible and make it easy to create reliable, useful benchmarks.

### Comparison with Criterion-rs

I intend Iai to be a complement to Criterion-rs, not a competitor. The two projects measure different
things in different ways and have different pros, cons, and limitations, so for most projects the
best approach is to use both.

Here's an overview of the important differences:
- Temporary Con: Right now, Iai is lacking many features of Criterion-rs, including reports and configuration of any kind.
    - The current intent is to add support to [Cargo-criterion] for configuring and reporting on Iai benchmarks.
- Pro: Iai can reliably detect much smaller changes in performance than Criterion-rs can.
- Pro: Iai can work reliably in noisy CI environments or even cloud CI providers like GitHub Actions or Travis-CI, where Criterion-rs cannot.
- Pro: Iai also generates profile output from the benchmark without further effort.
- Pro: Although Cachegrind adds considerable runtime overhead, running each benchmark exactly once is still usually faster than Criterion-rs' statistical measurements.
- Mixed: Because Iai can detect such small changes, it may report performance differences from changes to the order of functions in memory and other compiler details.
- Con: Iai's measurements merely correlate with wall-clock time (which is usually what you actually care about), where Criterion-rs measures it directly.
- Con: Iai cannot exclude setup code from the measurements, where Criterion-rs can.
- Con: Because Cachegrind does not measure system calls, IO time is not accurately measured.
- Con: Because Iai runs the benchmark exactly once, it cannot measure variation in the performance such as might be caused by OS thread scheduling or hash-table randomization.
- Limitation: Iai can only be used on platforms supported by Valgrind. Notably, this does not include Windows.

For benchmarks that run in CI (especially if you're checking for performance regressions in pull 
requests on cloud CI) you should use Iai. For benchmarking on Windows or other platforms that
Valgrind doesn't support, you should use Criterion-rs. For other cases, I would advise using both.
Iai gives more precision and scales better to larger benchmarks, while Criterion-rs allows for
excluding setup time and gives you more information about the actual time your code takes and how
strongly that is affected by non-determinism like threading or hash-table randomization. If you
absolutely need to pick one or the other though, Iai is probably the one to go with.

[Cargo-criterion]: https://github.com/bheisler/cargo-criterion

### Contributing

First, thank you for contributing.

One great way to contribute to Iai is to use it for your own benchmarking needs and report your experiences, file and comment on issues, etc.

Code or documentation improvements in the form of pull requests are also welcome. If you're not
sure what to work on, try checking the 
[Beginner label](https://github.com/bheisler/iai/issues?q=is%3Aissue+is%3Aopen+label%3ABeginner).

If your issues or pull requests have no response after a few days, feel free to ping me (@bheisler).

For more details, see the [CONTRIBUTING.md file](https://github.com/bheisler/iai/blob/master/CONTRIBUTING.md).

### Compatibility Policy

Iai supports the last three stable minor releases of Rust. At time of
writing, this means Rust 1.48 or later. Older versions may work, but are not tested or guaranteed.

Currently, the oldest version of Rust believed to work is 1.48. Future versions of Iai may
break support for versions older than 3-versions-ago, and this will not be considered a breaking change. If you
require Iai to work on old versions of Rust, you will need to stick to a
specific patch version of Iai.

### Maintenance

Iai was originally written and is maintained by Brook Heisler (@bheisler)

### License

Iai is dual licensed under the Apache 2.0 license and the MIT license.
