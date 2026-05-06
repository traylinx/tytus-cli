# Phase 1 Windows compile and stub inventory

**Status:** Phase 1 evidence seed.  
**Date:** 2026-05-06.  
**Branch:** `feature/tytus-os-multiplatform-phase0`.  
**Scope:** `services/tytus-cli`.

## Local command attempted

```bash
cargo check --workspace --target x86_64-pc-windows-msvc
```

Local result on Sebastian's macOS host is not authoritative for code correctness because `/usr/local/bin/cargo` and `/usr/local/bin/rustc` resolve to Homebrew Rust while `rustup` owns the Windows stdlib target. The failure is toolchain setup, not product code:

```text
error[E0463]: can't find crate for `core`
= note: the `x86_64-pc-windows-msvc` target may not be installed
```

Phase 1 CI adds a real `windows-latest` job. That job is the source of truth for Windows compile failures.

## Known Windows blockers from source inventory

| Area | Evidence | Phase owner |
|---|---|---|
| Daemon lifecycle | `cli/src/daemon_windows.rs` states the tray/daemon runtime is still Unix-socket based. | Phase 2 |
| IPC | `tray/src/socket.rs`, `cli/src/daemon.rs`, and tray comments rely on `/tmp/tytus` and Unix socket behavior. | Phase 2 |
| Single instance | `tray/src/single_instance.rs` uses `/tmp/tytus`, Unix permissions, and `libc::kill`. | Phase 2 |
| Process liveness/reap | `cli/src/tunnel_reap.rs` uses `/tmp/tytus`, Unix pidfiles, `ps`, and `libc::kill`. | Phase 2/4 |
| Tray launcher | `tray/src/launcher.rs` writes shell scripts under `/tmp/tytus` and opens terminal commands. | Phase 2/5 |
| Transfer log | `cli/src/transfer.rs` defaults to `/tmp/tytus/transfers.log` and uses Unix raw-fd locking. | Phase 2 |
| Tunnel | `tunnel/src/wireguard.rs` has incomplete Windows route/service setup per sprint truth. | Phase 4 |

## Expected CI behavior

The new `PR truth` workflow runs:

```bash
cargo check --workspace --all-targets
cargo test --workspace --all-targets
```

on `windows-latest`. Red Windows CI is acceptable during Phase 1 only if the failure contains exact file/error evidence. After Phase 2, red platform CI is no longer an allowed baseline.

## First Phase 2 worklist generated from this inventory

1. Implement the platform runtime path module.
2. Replace `/tmp/tytus` assumptions with platform runtime dirs.
3. Move daemon/tray IPC to the localhost HTTP + token + `control.json` contract.
4. Add Windows process/service liveness abstraction.
5. Replace Unix pidfile/`libc::kill` reaping with platform process APIs.
6. Keep legacy Unix socket reads only as macOS/Linux migration shims.
