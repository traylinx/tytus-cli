# Phase 1 TytusOS dist embedding inventory

**Status:** Phase 1 evidence.  
**Date:** 2026-05-06.

## Source/build relationship

TytusOS source lives in a sibling repo:

```text
/Users/sebastian/Projects/makakoo/api/ProjectWannolot/services/tytus-os
```

`tytus-tray` vendors the built web shell at:

```text
/Users/sebastian/Projects/makakoo/api/ProjectWannolot/services/tytus-cli/tray/web/os
```

Phase 1 adds `scripts/sync-tytus-os-dist.sh` as the deterministic embedding tool.

## Commands

Build TytusOS from the monorepo root:

```bash
cd /Users/sebastian/Projects/makakoo/api/ProjectWannolot/services/tytus-os
npm ci
npm run build:packages
npm run typecheck
npm run build
npm run test
```

Do not run clean CI installs from `services/tytus-os/app`; the root lockfile/workspace wiring is required for `@tytus/host-api`, `@tytus/app-forge`, and the bundled app packages.

Sync into tray:

```bash
cd /Users/sebastian/Projects/makakoo/api/ProjectWannolot/services/tytus-cli
TYTUS_OS_SOURCE=../tytus-os scripts/sync-tytus-os-dist.sh --sync
```

Check drift:

```bash
TYTUS_OS_SOURCE=../tytus-os scripts/sync-tytus-os-dist.sh --check
```

## Current local evidence

The vendored bundle must be generated from a clean TytusOS commit/ref using root workspace scripts. A build from a dirty sibling checkout is invalid because CI checks out the remote ref and will compare against that clean output.

Current embedded tree hash:

```text
1c5dfa71c8e0f7c5ae59bbf92561be35f02a2401f28a4da14a90b4213dcda0df
```

The hash is written to:

```text
tray/web/os.dist.sha256
```

## CI truth

The new `PR truth` workflow checks out `traylinx/tytus-os` at `feature/tytus-forge-mvp`, builds it, then runs the drift check against the vendored `tray/web/os` bundle.

The release workflow builds TytusOS from the requested `tytus_os_ref`, syncs the dist into the release artifact workspace, then builds all Rust binaries. Release artifacts therefore embed the built TytusOS dist deterministically rather than relying on a stale manual copy.
