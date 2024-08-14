#![cfg_attr(feature = "real_blackbox", feature(test))]

#[cfg(feature = "real_blackbox")]
extern crate test;

use cfg_if::cfg_if;
use std::{
    collections::HashMap,
    env::args,
    fs::File,
    io::{BufRead, BufReader},
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

#[cfg(feature = "macro")]
pub use iai_macro::iai;

mod macros;

/// A function that is opaque to the optimizer, used to prevent the compiler from
/// optimizing away computations in a benchmark.
///
/// This variant is backed by the (unstable) test::black_box function.
#[cfg(feature = "real_blackbox")]
pub fn black_box<T>(dummy: T) -> T {
    test::black_box(dummy)
}

/// A function that is opaque to the optimizer, used to prevent the compiler from
/// optimizing away computations in a benchmark.
///
/// This variant is stable-compatible, but it may cause some performance overhead
/// or fail to prevent code from being eliminated.
#[cfg(not(feature = "real_blackbox"))]
pub fn black_box<T>(dummy: T) -> T {
    unsafe {
        let ret = std::ptr::read_volatile(&dummy);
        std::mem::forget(dummy);
        ret
    }
}

fn check_valgrind() -> bool {
    let result = Command::new("valgrind")
        .arg("--tool=cachegrind")
        .arg("--version")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();

    match result {
        Err(e) => {
            println!("Unexpected error while launching valgrind. Error: {}", e);
            false
        }
        Ok(status) => {
            if status.success() {
                true
            } else {
                println!("Failed to launch valgrind. Error: {}. Please ensure that valgrind is installed and on the $PATH.", status);
                false
            }
        }
    }
}

fn get_arch() -> String {
    let output = Command::new("uname")
        .arg("-m")
        .stdout(Stdio::piped())
        .output()
        .expect("Failed to run `uname` to determine CPU architecture.");

    String::from_utf8(output.stdout)
        .expect("`-uname -m` returned invalid unicode.")
        .trim()
        .to_owned()
}

fn basic_valgrind() -> Command {
    Command::new("valgrind")
}

// Invoke Valgrind, disabling ASLR if possible because ASLR could noise up the results a bit
cfg_if! {
    if #[cfg(target_os = "linux")] {
        fn valgrind_without_aslr(arch: &str) -> Command {
            let mut cmd = Command::new("setarch");
            cmd.arg(arch)
                .arg("-R")
                .arg("valgrind");
            cmd
        }
    } else if #[cfg(target_os = "freebsd")] {
        fn valgrind_without_aslr(_arch: &str) -> Command {
            let mut cmd = Command::new("proccontrol");
            cmd.arg("-m")
                .arg("aslr")
                .arg("-s")
                .arg("disable");
            cmd
        }
    } else {
        fn valgrind_without_aslr(_arch: &str) -> Command {
            // Can't disable ASLR on this platform
            basic_valgrind()
        }
    }
}

fn run_bench(
    arch: &str,
    executable: &str,
    i: isize,
    name: &str,
    allow_aslr: bool,
) -> (CachegrindStats, Option<CachegrindStats>) {
    let output_file = PathBuf::from(format!("target/iai/cachegrind.out.{}", name));
    let old_file = output_file.with_file_name(format!("cachegrind.out.{}.old", name));
    std::fs::create_dir_all(output_file.parent().unwrap()).expect("Failed to create directory");

    if output_file.exists() {
        // Already run this benchmark once; move last results to .old
        std::fs::copy(&output_file, &old_file).unwrap();
    }

    let mut cmd = if allow_aslr {
        basic_valgrind()
    } else {
        valgrind_without_aslr(arch)
    };
    let status = cmd
        .arg("--tool=cachegrind")
        // Set some reasonable cache sizes. The exact sizes matter less than having fixed sizes,
        // since otherwise cachegrind would take them from the CPU and make benchmark runs
        // even more incomparable between machines.
        .arg("--cache-sim=yes")
        .arg("--I1=32768,8,64")
        .arg("--D1=32768,8,64")
        .arg("--LL=8388608,16,64")
        .arg(format!("--cachegrind-out-file={}", output_file.display()))
        .arg(executable)
        .arg("--iai-run")
        .arg(i.to_string())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .expect("Failed to run benchmark in cachegrind");

    if !status.success() {
        panic!(
            "Failed to run benchmark in cachegrind. Exit code: {}",
            status
        );
    }

    let new_stats = parse_cachegrind_output(&output_file);
    let old_stats = if old_file.exists() {
        Some(parse_cachegrind_output(&old_file))
    } else {
        None
    };

    (new_stats, old_stats)
}

fn parse_cachegrind_output(file: &Path) -> CachegrindStats {
    let mut events_line = None;
    let mut summary_line = None;

    let file_in = File::open(file).expect("Unable to open cachegrind output file");

    for line in BufReader::new(file_in).lines() {
        let line = line.unwrap();
        if let Some(line) = line.strip_prefix("events: ") {
            events_line = Some(line.trim().to_owned());
        }
        if let Some(line) = line.strip_prefix("summary: ") {
            summary_line = Some(line.trim().to_owned());
        }
    }

    match (events_line, summary_line) {
        (Some(events), Some(summary)) => {
            let events: HashMap<_, _> = events
                .split_whitespace()
                .zip(summary.split_whitespace().map(|s| {
                    s.parse::<u64>()
                        .expect("Unable to parse summary line from cachegrind output file")
                }))
                .collect();

            CachegrindStats {
                instruction_reads: events["Ir"],
                instruction_l1_misses: events["I1mr"],
                instruction_cache_misses: events["ILmr"],
                data_reads: events["Dr"],
                data_l1_read_misses: events["D1mr"],
                data_cache_read_misses: events["DLmr"],
                data_writes: events["Dw"],
                data_l1_write_misses: events["D1mw"],
                data_cache_write_misses: events["DLmw"],
            }
        }
        _ => panic!("Unable to parse cachegrind output file"),
    }
}

#[derive(Clone, Debug)]
struct CachegrindStats {
    instruction_reads: u64,
    instruction_l1_misses: u64,
    instruction_cache_misses: u64,
    data_reads: u64,
    data_l1_read_misses: u64,
    data_cache_read_misses: u64,
    data_writes: u64,
    data_l1_write_misses: u64,
    data_cache_write_misses: u64,
}
impl CachegrindStats {
    pub fn ram_accesses(&self) -> u64 {
        self.instruction_cache_misses + self.data_cache_read_misses + self.data_cache_write_misses
    }
    pub fn summarize(&self) -> CachegrindSummary {
        let ram_hits = self.ram_accesses();
        let l3_accesses =
            self.instruction_l1_misses + self.data_l1_read_misses + self.data_l1_write_misses;
        let l3_hits = l3_accesses - ram_hits;

        let total_memory_rw = self.instruction_reads + self.data_reads + self.data_writes;
        let l1_hits = total_memory_rw - (ram_hits + l3_hits);

        CachegrindSummary {
            l1_hits,
            l3_hits,
            ram_hits,
        }
    }

    #[rustfmt::skip]
    pub fn subtract(&self, calibration: &CachegrindStats) -> CachegrindStats {
        CachegrindStats {
            instruction_reads: self.instruction_reads.saturating_sub(calibration.instruction_reads),
            instruction_l1_misses: self.instruction_l1_misses.saturating_sub(calibration.instruction_l1_misses),
            instruction_cache_misses: self.instruction_cache_misses.saturating_sub(calibration.instruction_cache_misses),
            data_reads: self.data_reads.saturating_sub(calibration.data_reads),
            data_l1_read_misses: self.data_l1_read_misses.saturating_sub(calibration.data_l1_read_misses),
            data_cache_read_misses: self.data_cache_read_misses.saturating_sub(calibration.data_cache_read_misses),
            data_writes: self.data_writes.saturating_sub(calibration.data_writes),
            data_l1_write_misses: self.data_l1_write_misses.saturating_sub(calibration.data_l1_write_misses),
            data_cache_write_misses: self.data_cache_write_misses.saturating_sub(calibration.data_cache_write_misses),
        }
    }
}

#[derive(Clone, Debug)]
struct CachegrindSummary {
    l1_hits: u64,
    l3_hits: u64,
    ram_hits: u64,
}
impl CachegrindSummary {
    fn cycles(&self) -> u64 {
        // Uses Itamar Turner-Trauring's formula from https://pythonspeed.com/articles/consistent-benchmarking-in-ci/
        self.l1_hits + (5 * self.l3_hits) + (35 * self.ram_hits)
    }
}

/// Custom-test-framework runner. Should not be called directly.
#[doc(hidden)]
pub fn runner(benches: &[&(&'static str, fn())]) {
    let mut args_iter = args();
    let executable = args_iter.next().unwrap();

    if let Some("--iai-run") = args_iter.next().as_deref() {
        // In this branch, we're running under cachegrind, so execute the benchmark as quickly as
        // possible and exit
        let index: isize = args_iter.next().unwrap().parse().unwrap();

        // -1 is used as a signal to do nothing and return. By recording an empty benchmark, we can
        // subtract out the overhead from startup and dispatching to the right benchmark.
        if index == -1 {
            return;
        }

        let index = index as usize;

        (benches[index].1)();
        return;
    }

    // Otherwise we're running normally, under cargo
    if !check_valgrind() {
        return;
    }

    let arch = get_arch();

    let allow_aslr = std::env::var_os("IAI_ALLOW_ASLR").is_some();

    let (calibration, old_calibration) =
        run_bench(&arch, &executable, -1, "iai_calibration", allow_aslr);

    for (i, (name, _func)) in benches.iter().enumerate() {
        println!("{}", name);
        let (stats, old_stats) = run_bench(&arch, &executable, i as isize, name, allow_aslr);
        let (stats, old_stats) = (
            stats.subtract(&calibration),
            match (&old_stats, &old_calibration) {
                (Some(old_stats), Some(old_calibration)) => {
                    Some(old_stats.subtract(old_calibration))
                }
                _ => None,
            },
        );

        fn signed_short(n: f64) -> String {
            let n_abs = n.abs();

            if n_abs < 10.0 {
                format!("{:+.6}", n)
            } else if n_abs < 100.0 {
                format!("{:+.5}", n)
            } else if n_abs < 1000.0 {
                format!("{:+.4}", n)
            } else if n_abs < 10000.0 {
                format!("{:+.3}", n)
            } else if n_abs < 100000.0 {
                format!("{:+.2}", n)
            } else if n_abs < 1000000.0 {
                format!("{:+.1}", n)
            } else {
                format!("{:+.0}", n)
            }
        }

        fn percentage_diff(new: u64, old: u64) -> String {
            if new == old {
                return " (No change)".to_owned();
            }

            let new: f64 = new as f64;
            let old: f64 = old as f64;

            let diff = (new - old) / old;
            let pct = diff * 100.0;

            format!(" ({:>+6}%)", signed_short(pct))
        }

        println!(
            "  Instructions:     {:>15}{}",
            stats.instruction_reads,
            match &old_stats {
                Some(old) => percentage_diff(stats.instruction_reads, old.instruction_reads),
                None => "".to_owned(),
            }
        );
        let summary = stats.summarize();
        let old_summary = old_stats.map(|stat| stat.summarize());
        println!(
            "  L1 Accesses:      {:>15}{}",
            summary.l1_hits,
            match &old_summary {
                Some(old) => percentage_diff(summary.l1_hits, old.l1_hits),
                None => "".to_owned(),
            }
        );
        println!(
            "  L2 Accesses:      {:>15}{}",
            summary.l3_hits,
            match &old_summary {
                Some(old) => percentage_diff(summary.l3_hits, old.l3_hits),
                None => "".to_owned(),
            }
        );
        println!(
            "  RAM Accesses:     {:>15}{}",
            summary.ram_hits,
            match &old_summary {
                Some(old) => percentage_diff(summary.ram_hits, old.ram_hits),
                None => "".to_owned(),
            }
        );
        println!(
            "  Estimated Cycles: {:>15}{}",
            summary.cycles(),
            match &old_summary {
                Some(old) => percentage_diff(summary.cycles(), old.cycles()),
                None => "".to_owned(),
            }
        );
        println!();
    }
}
