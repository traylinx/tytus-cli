# Phase 1 workflow dry-run notes

**Status:** Phase 1 evidence.
**Date:** 2026-05-06.

## Cross-repo checkout

`traylinx/tytus-os` is reachable through unauthenticated `git ls-remote` from this machine:

```text
git ls-remote https://github.com/traylinx/tytus-os.git HEAD
4067ad254888f6849442f58cd1294cbd780b73c1 HEAD
```

Because the repo is readable without a token, the PR workflow does not add a custom PAT for the `actions/checkout` of `traylinx/tytus-os`.

If the repo becomes private, the workflow must add an app/PAT secret before CI can stay green.

## Release determinism

PR truth may default to `feature/tytus-forge-mvp` because it is a moving integration branch and the point is early breakage detection.

Release builds must not default to that branch. `release.yml` requires `tytus_os_ref` and rejects `main` or `feature/*` branch names. Use a TytusOS tag or commit SHA for release rebuild determinism.

## First remote dry run required before real tag

Before tagging a real production release, run:

1. `PR truth` on this branch.
2. `Release` on a throwaway `-rc` tag with immutable `tytus_os_ref`.

That dry run proves:

- GitHub can checkout `traylinx/tytus-os`.
- Ubuntu rebuild of TytusOS byte-matches committed `tray/web/os` for PR drift check.
- Windows builds/package step runs under Bash.
- Windows `tytus-tray.exe` compile status becomes real CI evidence.

Until that remote dry run passes, Phase 1 is locally complete but remote CI truth is still pending.

## TytusOS workspace command shape

TytusOS is an npm workspaces monorepo. CI must run npm commands from `tytus-os/`, not `tytus-os/app/`.

Required sequence:

```bash
cd tytus-os
npm ci
npm run build:packages
npm run typecheck
npm run build
npm run test
```

The explicit `build:packages` before `typecheck` prevents clean-clone failures where the app imports `@tytus/host-api` or bundled app packages before their declaration/build outputs exist.
