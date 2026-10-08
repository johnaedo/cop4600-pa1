## PA1 (Redesigned): Real-Time Process Scheduling in Rust

## Introduction

You are tasked with implementing three process scheduling algorithms — FIFO
(First In, First Out), preemptive SJF (Shortest Job First), and Round Robin —
in Rust. Unlike a typical scheduling assignment, your scheduler does not just
track process state in a data structure: it must **enact its scheduling
decisions on real OS processes**, using the same primitives a real operating
system's scheduler relies on (process creation, signals, process reaping, and
a monotonic clock).

You'll also use a generative AI coding agent as part of this assignment —
to help you diagnose and fix concurrency bugs in a provided reference implementation. Verifying that a fix is *actually* correct requires running the program, not just reading the code — which is exactly the skill this component is meant to build.

## Learning Objectives

- Use `fork`/`exec`, signals, `waitpid`, and a monotonic clock correctly to
  control real processes from Rust.
- Understand scheduling as a separation of concerns: **policy** (which
  process runs next — your FIFO/SJF/RR logic) versus **mechanism** (how that
  decision is enacted on the OS (e.g. signals)).
- Practice using an AI coding agent as a debugging collaborator for
  concurrency bugs, and learn to verify its claims against real execution
  rather than trusting confident-sounding explanations.

## What's Provided

- A Docker development image with the Rust toolchain and the `nix` and `libc` crates preinstalled.
- `workload.rs`: a `worker process` that consumes a specified
  amount of real **CPU** time (not wall-clock time).  It deliberately measures `CLOCK_PROCESS_CPUTIME_ID` rather than a wall clock: CPU time doesn't advance while a process is stopped, so a process you `SIGSTOP` and later `SIGCONT` still ends up consuming exactly the burst you asked for, regardless of how many times it gets preempted. 
- `process_control.rs`: a scaffold module with safe wrappers around
  `fork`/`execvp`, `SIGSTOP`/`SIGCONT`, `waitpid`, and clock reads. This
  gives you the syscall building blocks; the scheduling policy and the
  ordering/race reasoning are still yours to get right.  Note, however, that there are two concurrency bugs in this code.  To earn full credit, you must identify and correct the issue.  You may use AI (see below).
- Sample `.in` test files for each algorithm, plus two malformed inputs for
  testing your error handling and one designed to leave a process
  unfinished.
- `main.rs`: This is where you will implement the scheduling functions.  Note that while this code is only *missing* the policy implementations, it also has one subtle bug that will need to be fixed (more on this below).

## Missing Functions

The FCFS, SJF, and Round Robin are intentionally left blank.  It is up to you to fill them out:

I'll give you a hint on FCFS:  It doesn't do much.  It's one line of code.
```rust
fn fcfs(
    ci: usize,
    _queue: &mut Vec<usize>,
    _procs: &[RuntimeProc],
    _config: &Config,
    _rr_elapsed: &mut u64,
) -> Option<usize> {
    // YOUR IMPLEMENTATION GOES HERE
}

fn sjf(
    ci: usize,
    queue: &mut Vec<usize>,
    procs: &[RuntimeProc],
    _config: &Config,
    _rr_elapsed: &mut u64,
) -> Option<usize> {
    // YOUR IMPLEMENTATION GOES HERE
}

fn rr(
    ci: usize,
    queue: &mut Vec<usize>,
    procs: &[RuntimeProc],
    config: &Config,
    rr_elapsed: &mut u64,
) -> Option<usize> {
    // YOUR IMPLEMENTATION GOES HERE
}
```

## System Requirements

Your scheduler's decisions must be carried out on real child processes, not
simulated in memory:

| Scheduling concept          | Must be implemented as                                                                                                                                                                                                                                                               |
| --------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| A "process"                 | A real forked child running `workload <ticks>`                                                                                                                                                                                                                                       |
| "Selecting" a process       | Sending it `SIGCONT`                                                                                                                                                                                                                                                                 |
| "Preempting" a process      | Sending it `SIGSTOP`                                                                                                                                                                                                                                                                 |
| Detecting "finished"        | `waitpid` returning that PID as exited                                                                                                                                                                                                                                               |
| Every timestamp you print   | A real read of a monotonic clock at the moment of the event                                                                                                                                                                                                                          |
| Cleanup at the end of a run | Every process you spawned must be gone from the process table before your program exits — including ones still running or stopped because they didn't finish within `runfor`. `SIGKILL` alone is not enough: an unreaped killed child is a zombie that remains in the process table. |

Your submission's syscall trace will be checked automatically for the
presence of process creation (`clone`/`fork`), `execve`, signal delivery
(`kill`/`tgkill`), and process reaping (`wait4`/`waitid`). A submission that
never calls these — e.g. one that fakes timing with `thread::sleep` and an
in-memory clock — will not receive credit for this requirement, regardless
of whether its printed output looks correct.

## Input File Format

`runfor` and `burst` values are in **deciseconds (tenths of a second) of real wall-clock time**, not
abstract simulated ticks

```
processcount 3
runfor 20
use sjf
process name A arrival 0 burst 5
process name B arrival 1 burst 4
process name C arrival 4 burst 2
end
```

| directive      | definition                                                          |
| -------------- | ------------------------------------------------------------------- |
| `processcount` | Number of processes in the list                                     |
| `runfor`       | How many deciseconds of real time to run for                        |
| `use`          | Algorithm: `fcfs`, `sjf`, or `rr`                                   |
| `quantum`      | (rr only) quantum length, in deciseconds                            |
| `process`      | `name`, `arrival`, `burst` — same meaning as before, in deciseconds |
| `end`          | End-of-file marker                                                  |

### Validation requirements

- Missing required parameter → `Error: Missing parameter <parameter>.`
- `use rr` with no `quantum` → `Error: Missing quantum parameter when use is 'rr'`
- No input file argument → `Usage: scheduler-gpt <input file>`

## Output File Format

*Note:* Because timestamps come from a real
clock rather than a deterministic simulation, they will show small jitter
(scheduling latency, signal delivery delay) — this is expected and is
graded with tolerance, not exact match. Event **ordering**, however, is
still fully deterministic (it's produced by your policy logic) and is
graded exactly.  The values below are just examples.

```
3 processes
Using preemptive Shortest Job First
Time   0 : A arrived
Time   0 : A selected (burst   5)
Time   1 : B arrived
Time   4 : C arrived
Time   5 : A finished
Time   5 : C selected (burst   2)
Time   7 : C finished
Time   7 : B selected (burst   4)
Time  11 : B finished
Time  11 : Idle
...
Finished at time  20

A wait   0 turnaround   5  response 0
B wait   6 turnaround  10  response 6
C wait   1 turnaround   3
```

Unfinished processes at the end of `runfor` are listed as:

```
P1 did not finish
```

## The AI Component

You will receive `main.rs` and `process-control.rs` files.  In addition to completing the scheduling policies, you must track down and fix 3 concurrency bugs left in these files.  None of these bugs show up in the output files from the given tests, and none of them are caught by the autograder, syscall-usage, or error-handling checks. 

1. **A stop/resume race**: the
   child is given a brief head start before `SIGSTOP` is sent, on the
   reasoning that "it needs a moment to exec first." During that window
   it's actually running the real workload, burning real CPU time no
   scheduling decision accounted for. This one is genuinely subtle: at
   this assignment's tick granularity (100ms) it mostly doesn't shift any
   single process's timing past the grading tolerance, so it will not
   reliably show up on the provided sample tests. Finding it requires
   either reasoning about the code or constructing a stress test — e.g. many short-burst processes arriving close together — where the effect compounds enough to notice.
2. **A missing reap**: This one is fully deterministic and easy to demonstrate:
   run the scheduler against `tests/unfinished.in` (one process with a
   burst longer than `runfor`) and check the process table immediately
   afterward — `ps aux | grep defunct` will show a real `<defunct>` zombie
   left behind, even though `unfinished.out` is byte-for-byte correct.
3. **A mishandled interrupted syscall**: the blocking wait for a finishing process is
   implemented as a single raw `libc::waitpid` call. Because the scheduler is sending signals to sibling processes throughout the run, this occasionally gets interrupted —
   and when it does, the process is logged as "finished" without its
   exit status ever actually being collected. This one is intermittent
   by nature (it didn't reproduce on every run during testing, and was
   more likely on the busier `rr.in` case than `fcfs.in`) — which is
   itself the point: running it once, seeing clean output, and
   declaring it fixed doesn't confirm anything.

Your task: use an AI coding agent to help locate and fix these bugs, then
submit:

- **The agent transcript or exported session log**, showing your diagnostic
  process (not just a final "fixed" message).
- **A short written reflection (250–300 words)** identifying at
  least one point where the agent proposed a fix that looked correct but
  didn't actually resolve the bug, and describing how you confirmed the
  real fix — e.g. repeated runs, `strace`, a stress-test loop that
  triggers the race more reliably, etc. A reflection that only describes
  bugs the agent got right on the first try will not receive full credit
  for this component — the point is learning to verify, not to transcribe
  a clean success story.

## Deliverables

- `main.rs` implementing FCFS, SJF, and RR per the specs above.
- The AI agent transcript/log file.
- The written reflection (.pdf or .docx).

This can be completed individually or in small teams.

## Grading Rubric (100 points)

| Component                                        | Points  | How it's graded             |
| ------------------------------------------------ | ------- | --------------------------- |
| Correctness — FCFS                               | 15      | Automated (`autograder.py`) |
| Correctness — SJF                                | 15      | Automated                   |
| Correctness — RR                                 | 15      | Automated                   |
| Real-syscall usage                               | 15      | Automated (`strace` check)  |
| Error handling (malformed/missing input)         | 10      | Automated                   |
| AI debugging component (transcript + reflection) | 20      | TA-graded, rubric checklist |
| Code quality (comments, structure)               | 10      | TA-graded                   |
| **Total**                                        | **100** |                             |

