# Docker Environment — Setup & Usage

This gives you an identical Linux environment for
PA1: Real-Time Process Scheduling in Rust, with the Rust toolchain, the `nix`/
`libc` crates, and `strace`

## Project layout expected by these files

```
pa1/
├── Dockerfile
├── docker-compose.yml
├── Assignment_Spec.md
├── src/
	├── main.rs
│   ├── process_control.rs
│   └── workload.rs
├── tests/
│   ├── fcfs.in, sjf.in, rr.in
│   └── missing_processcount.in, missing_quantum.in
└── main.rs
```

## Setup (all platforms)

1. Install Docker:
   - **Windows**: install [Docker Desktop](https://www.docker.com/products/docker-desktop/), and during setup choose the **WSL2 backend** when prompted (this is the default on current versions). You do not need a separate WSL Linux distro installed yourself — Docker Desktop manages it.
   - **macOS**: install [Docker Desktop](https://www.docker.com/products/docker-desktop/) (Intel or Apple Silicon build, matching your Mac). No other setup needed.
   - **Linux**: install Docker Engine and the Compose plugin for your distribution (e.g. on Ubuntu/Debian: `sudo apt-get install docker.io docker-compose-plugin`). Add yourself to the `docker` group so you don't need `sudo` for every command: `sudo usermod -aG docker $USER`, then log out and back in.
2. Confirm it works: open a terminal and run `docker --version` and `docker compose version`. Both should print a version number.

## Building the image (do this first, every time you pull new scaffold files)

From inside your `pa1/` project directory:

```
docker compose build
```

This only needs to be re-run when `Dockerfile` changes (e.g. if a new
scaffold update requires a different crate version). Ordinary code changes
don't require a rebuild.

## Day-to-day development

Open an interactive shell in the container:

```
docker compose run --rm dev
```

You're now in a Bash shell inside the container, at `/workspace`, which is
your project directory. Anything you edit on your host (VS Code, vim,
whatever) is immediately visible inside the container, and vice versa.
From here:

```
cargo build                # compiles your scheduler
./target/debug/scheduler-gpt tests/sjf.in
cat tests/sjf.out
strace -f -c ./target/debug/scheduler-gpt tests/rr.in   # see your own syscall trace
```

Type `exit` to leave the container shell; `--rm` means the container is
cleaned up automatically (your files, which live on your host via the
volume mount, are untouched).

### Platform-specific notes

- **Windows**: run these `docker compose` commands from PowerShell, Command Prompt, or a WSL terminal — all work identically. If you keep your project files inside the WSL filesystem (e.g. under `\\wsl$\...` or opened via `code .` from WSL), file-watching and build performance will be noticeably better than a project on the Windows `C:\` drive.
- **macOS**: the first time you run `docker compose run`, Docker Desktop may prompt for permission to share your project folder — allow it, or the volume mount will fail silently. Apple Silicon (M-series) Macs run this image natively; no configuration needed.
- **Linux**: works directly with no VM layer. If you hit a "permission denied" error on files created inside the container, it's almost always a UID mismatch — the image creates its internal user with UID 1000, which matches the default first user on most Linux distros, but if your host UID differs, run `id -u` on your host and rebuild with `docker build --build-arg UID=$(id -u) .` after adding a `UID` build arg to the Dockerfile, or simply `sudo chown -R $USER:$USER .` after running the container.

## Troubleshooting

- **`strace` reports permission errors inside the container**: confirm `docker-compose.yml` still has `cap_add: [SYS_PTRACE]` under the service you're running — this is required and is already set for both `dev` and `grader` above.
- **Build is slow every single time**: check that `docker compose build` actually completed (look for the `warmup` cargo step in the output) rather than being skipped due to a cache miss on every layer — this can happen if the Dockerfile is edited frequently. It only needs to run once until the Dockerfile changes again.
- **Windows: mounted files show as read-only or with wrong line endings**: this happens if Git is configured to convert line endings on checkout (`core.autocrlf=true`). Run `git config core.autocrlf false` in the project repo before checking it out, since Rust source files should keep LF endings inside the Linux container.
