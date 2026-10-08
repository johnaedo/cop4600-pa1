// workload.rs -- provided scaffold. Do not modify.
//
// A minimal CPU-bound workload. It is spawned as a child process by the
// student's scheduler; SIGSTOP/SIGCONT sent by the parent control when it
// actually gets CPU time.
//
// Usage: workload <ticks>
//   <ticks> is in deciseconds (tenths of a second) of *actual CPU time
//   consumed*, not wall-clock time. This distinction matters: this
//   process measures CLOCK_PROCESS_CPUTIME_ID, which only advances while
//   the process is actually running on a CPU. If it measured wall-clock
//   time instead, a process that gets SIGSTOPped and resumed later would
//   finish "early" relative to its intended burst, because wall-clock
//   time keeps advancing while the process is stopped even though it did
//   no work during that interval. Using CPU time means this binary
//   correctly reports (via its exit timing) exactly <ticks> deciseconds
//   of real execution, regardless of how many times it gets preempted.

use libc::{clock_gettime, timespec, CLOCK_PROCESS_CPUTIME_ID};
use std::env;
use std::mem::MaybeUninit;
use std::process::exit;

fn cpu_time_ns() -> i64 {
    unsafe {
        let mut ts = MaybeUninit::<timespec>::uninit();
        clock_gettime(CLOCK_PROCESS_CPUTIME_ID, ts.as_mut_ptr());
        let ts = ts.assume_init();
        ts.tv_sec as i64 * 1_000_000_000 + ts.tv_nsec as i64
    }
}

fn main() {
    let ticks: i64 = match env::args().nth(1).and_then(|s| s.parse().ok()) {
        Some(t) => t,
        None => {
            eprintln!("Usage: workload <ticks>");
            exit(1);
        }
    };

    let target_ns = ticks * 100_000_000; // deciseconds -> nanoseconds
    let start = cpu_time_ns();

    let mut counter: u64 = 0;
    while cpu_time_ns() - start < target_ns {
        counter = counter.wrapping_add(1);
    }

    // Prevents the optimizer from eliminating the loop entirely.
    if counter == u64::MAX {
        println!("unreachable");
    }

    exit(0);
}
