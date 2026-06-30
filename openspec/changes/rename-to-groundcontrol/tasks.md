## 1. Package Rename

- [x] 1.1 Rename package name in Cargo.toml from `sigground` to `groundcontrol`
- [x] 1.2 Update doc comment and clap command name in src/main.rs
- [x] 1.3 Rename `SIGGROUND_MARKER` constant and update init print message
- [x] 1.4 Update README.md — replace all `sigground` references with `groundcontrol`

## 2. Verification

- [x] 2.1 Verify `cargo build` produces `groundcontrol` binary
- [x] 2.2 Verify `cargo test` passes
- [x] 2.3 Verify `grep -c sigground src/ README.md` returns 0
