//! Contains macros which together define a benchmark harness that can be used
//! in place of the standard benchmark harness. This allows the user to run
//! Iai benchmarks with `cargo bench`.

/// Macro which expands to a benchmark harness.
///
/// Currently, using Iai requires disabling the benchmark harness
/// generated automatically by rustc. This can be done like so:
///
/// ```toml
/// [[bench]]
/// name = "my_bench"
/// harness = false
/// ```
///
/// In this case, `my_bench` must be a rust file inside the 'benches' directory,
/// like so:
///
/// `benches/my_bench.rs`
///
/// Since we've disabled the default benchmark harness, we need to add our own:
///
/// ```ignore
/// fn bench_method1() {
/// }
///
/// fn bench_method2() {
/// }
///
/// iai::main!(bench_method1, bench_method2);
/// ```
///
/// The `iai::main` macro expands to a `main` function which runs all of the
/// benchmarks in the given groups.
///
#[macro_export]
macro_rules! main {
    ( $( $func_name:ident ),+ $(,)* ) => {
        mod iai_wrappers {
            $(
                pub fn $func_name() {
                    let _ = $crate::black_box(super::$func_name());
                }
            )+
        }

        fn main() {

            let benchmarks : &[&(&'static str, fn())]= &[

                $(
                    &(stringify!($func_name), iai_wrappers::$func_name),
                )+
            ];

            $crate::runner(benchmarks);
        }
    }
}
