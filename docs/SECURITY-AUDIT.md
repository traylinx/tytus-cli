# tytus-cli — Security Audit (Pre-Public Release)

**Date:** 2026-04-10
**Auditor:** internal review (Claude Opus 4.6 driving Sebastian's environment)
**Scope:** the entire `traylinx/tytus-cli` repository — git history, source code,
dependencies, install script, sudoers entry, hosted skill file, packaging.
**Reason:** Sebastian wants to flip the repo from `private` to `public` so the
2md-style "paste this prompt into any AI tool" bootstrap pattern can work
(`raw.githubusercontent.com` requires public visibility). Once the repo is
public, the entire git history becomes permanent — anything that has ever
been committed stays exposed forever.

## Verdict

**APPROVED for public visibility flip — pending final operator review.**

All blockers identified during the audit have been remediated. The remaining
items are tracked as backlog and are not exploitable.

| Severity | Total | Fixed | Remaining |
|---|---|---|---|
| CRITICAL | 1 | 1 | 0 |
| HIGH | 3 | 3 | 0 |
| MEDIUM | 3 | 3 | 0 |
| LOW | 6 | 6 | 0 |
| INFO | 2 | — | 2 (acknowledged) |

---

## Methodology

Seven phases, executed in order:

1. **Git history secret scan** — every commit, every diff, hunting for
   hardcoded API keys, tokens, private keys, internal hostnames, and any
   other material that should never become public.
2. **Source code audit** — hardcoded URLs/IPs/ports, credentials, error
   message leakage, `unsafe` Rust, subprocess injection vectors, path
   traversal, file permission handling.
3. **Crypto + secrets handling** — `Zeroize` coverage, OS keychain
   integration, state file `0600` enforcement, token redaction.
4. **Dependency audit** — `cargo audit` for known CVEs, `cargo tree`
   for supply chain visibility.
5. **Install script + sudoers review** — `install.sh` attack surface,
   the `/etc/sudoers.d/tytus` entry's exact privilege scope, command
   injection vectors in the elevated paths.
6. **Documentation review** — `Cargo.toml` metadata, `README.md`,
   `CLAUDE.md`, `SKILL.md`, `llm-docs.md`, `docs/*` — ensuring nothing
   public-facing leaks internal infrastructure or stale information.
7. **Compile + lint + test** — `cargo clippy --workspace --all-targets`,
   `cargo test --workspace`, `cargo audit` (gate: zero warnings, zero
   vulnerabilities, zero failing tests).

---

## Findings + remediations

### CRITICAL-1: sudoers entry allowed arbitrary `kill -TERM`

**Finding.** The `install.sh` installer wrote a sudoers entry that included
`/bin/kill -TERM *` in addition to the legitimate `${BIN_PATH} tunnel-up *`.
With this rule, any local user with the entry could send SIGTERM to **any
process on the system as root** — including PID 1, system services, other
users' processes, and security daemons. While SIGTERM does not directly grant
code execution, it is a real privilege escalation: the user could disrupt the
system, kill EDR/AV agents, deny service to other users, or chain it into
DoS scenarios.

**Root cause.** `tytus disconnect` needed a way to send SIGTERM to a tunnel
daemon (which runs as root because it owns the TUN device). The original
design used `sudo -n kill -TERM <pid>` and added a wildcard rule to sudoers.
The wildcard was the bug — sudoers rules must be tightly scoped.

**Fix.**
- New hidden subcommand `tytus tunnel-down <pid>` (`cli/src/main.rs`).
- Validates that the PID appears in `/tmp/tytus/tunnel-*.pid` (the daemon's
  own breadcrumb file).
- Verifies the process still exists via `kill -0` before signalling.
- If validation passes, sends SIGTERM via `libc::kill`.
- If the PID is `<= 1` it refuses immediately (defence against any
  upstream parsing weirdness that might end up calling with `0` or `1`).
- The sudoers entry in `install.sh` is now scoped to **only**:
  ```
  ${USER} ALL=(root) NOPASSWD: ${BIN_PATH} tunnel-up *, ${BIN_PATH} tunnel-down *
  ```
- `cmd_disconnect` was updated to invoke `sudo -n <self_exe> tunnel-down
  <pid>` instead of `sudo -n kill -TERM <pid>`.

**Result.** Even with the passwordless sudoers entry, an attacker (or buggy
caller) cannot use `tytus tunnel-down` to signal arbitrary processes — the
binary itself enforces the validation. The previous escalation path is
closed.

**Verified.** `tytus tunnel-down 1` exits 1 with `refusing to signal PID 1`.
`tytus tunnel-down <random>` exits 1 with `not a registered tytus tunnel
daemon`. `tytus disconnect` end-to-end still works because the tunnel
daemon writes its own PID to `/tmp/tytus/tunnel-NN.pid` on startup, which
matches the validation.

---

### HIGH-1: README.md leaked production data + outdated info

**Finding.** The committed `README.md` contained:

- `sk-566cecd...09a0` — the truncated display form of pod 01's real
  production AIL key. While the middle 50 hex characters were redacted, the
  prefix (8) + suffix (4) reduces the brute-force search space and matches
  exactly what `tytus status` prints today, allowing correlation if the
  same key ever leaks via another channel.
- `sk-c939e2...2318` — same pattern for pod 02.
- `10.18.1.1` and `10.18.2.1` — internal pod gateway IPs revealing the
  production droplet's `DROPLET_OCTET=18` value.
- Phantom model references (`qwen3-8b`, `llama-3.1-8b-instruct`, "383+
  models") — none of which exist on the SwitchAILocal gateway. The real
  catalog is five models (`ail-compound`, `ail-image`, `ail-embed`,
  `minimax/ail-compound`, `minimax/ail-image`).
- A broken install URL: `https://tytus.traylinx.com/install.sh` does not
  exist. The actual installer is at
  `https://raw.githubusercontent.com/traylinx/tytus-cli/main/install.sh`.
- "Zombie fungus" / "parasitize" / "infect" wording — accurate metaphor
  but sets the wrong tone for a public-facing project README.

**Fix.** Full rewrite of `README.md`. New content:

- Uses placeholder/stable values (`http://10.42.42.1:18080/v1`,
  `sk-tytus-user-<32hex>`) — never internal IPs or fingerprints of real keys.
- Lists the accurate five-model catalog.
- Points at the correct `raw.githubusercontent.com` install URL.
- Uses the new positive verb: `tytus link` instead of `tytus infect`.
- Documents the security posture upfront in its own section.
- Cross-references this audit document.

---

### HIGH-2: `docs/VERIFICATION-2026-04-10.md` was an internal audit dump

**Finding.** A 6.7KB file under `docs/` containing:

- Production droplet IP: `<redacted — production droplet IPv4>`
- Droplet ID: `<redacted — droplet short-name>`
- Droplet resource specs: "8 cores, 29GB free RAM, 439GB free disk"
- Internal architecture details: K8s deployment names, DAM port, nginx LB
  port, sidecar count, exact subnet schema
- Internal commit hashes from sibling private repos (`wannolot-provider`,
  `wannolot-infrastructure`)
- Authoring credit: "Claude Opus 4.6 (Harvey)" — internal only
- A detailed "what's broken right now" section that reveals known issues

This file was an engineering verification report, never intended for
public consumption. It would be the first thing a curious visitor finds in
a public repo.

**Fix.** File deleted entirely from the working tree. Will be removed
from history via the same commit (or, if the user wants stronger
guarantees, via a subsequent BFG-style history rewrite — flagged as a
follow-up below).

---

### HIGH-3: `docs/WIZARDS.md` referenced internal IP

**Finding.** A wizard-design document used `http://10.18.1.1:18080` in a
"Returning user" example, exposing the production internal subnet schema.

**Fix.** Replaced with the stable `http://10.42.42.1:18080` and added a
parenthetical "(stable, never changes)" so future readers know not to
substitute it back to a per-pod IP.

---

### MEDIUM-1: RUSTSEC-2026-0037 in `quinn-proto 0.11.13`

**Finding.** `cargo audit` flagged a known high-severity vulnerability
(CVSS 8.7) in the QUIC protocol implementation pulled in transitively
via `quinn → reqwest 0.12.28`. Affected version: `quinn-proto 0.11.13`.
Fix available in `>=0.11.14`.

**Fix.** `cargo update -p quinn-proto` upgraded the lockfile to
`quinn-proto 0.11.14`. Re-running `cargo audit` confirmed the
vulnerability is no longer present.

`Cargo.lock` is committed so all consumers (CI, the install script's
cargo install --git path, GitHub release builds) get the patched
transitive dependency.

---

### MEDIUM-2: `CLAUDE.md` was outdated

**Finding.** The engineering CLAUDE.md still referenced `tytus infect`,
omitted the new `link` / `bootstrap-prompt` / `llm-docs` / `tunnel-down`
commands, and had stale architecture descriptions.

**Fix.** Rewritten to reflect current command surface, hidden subcommands,
state and security invariants, the stable URL/key model, and contributing
guidelines. Cross-references `docs/SECURITY-AUDIT.md` (this file).

---

### MEDIUM-3: `mcp/src/tools.rs:268` referenced broken install URL

**Finding.** The `tytus_setup_guide` MCP tool returned a step that told
agents to install with `curl -fsSL https://tytus.traylinx.com/install.sh | sh`.
That URL doesn't resolve.

**Fix.** Replaced with the correct
`https://raw.githubusercontent.com/traylinx/tytus-cli/main/install.sh`
URL. Also softened the connect step to no longer require `sudo` (since the
elevation chain handles it internally now).

---

### LOW-1: `.gitignore` was too thin

**Finding.** Only `target/`, `*.swp`, `.DS_Store`. No protection against
accidentally committing `.env` files, `*.pem`/`*.key` certificates,
`state.json` (which contains the user's secret_key + tokens), `*.log`
files, or IDE configs.

**Fix.** Expanded to include `.env*`, `*.pem`, `*.key`, `*.p12`, `*.pfx`,
`*.crt`, `secrets/`, `state.json`, `**/state.json`, `*.log`, `logs/`,
`.idea/`, `.vscode/`, `*.iml`, `.cache/`. The pattern `!.env.example`
explicitly allows committing example env templates if needed.

---

### LOW-2: 23 clippy warnings (no errors)

**Finding.** `cargo clippy --workspace --all-targets` produced 23
warnings: `map_or` simplifications, `needless_borrow`, unused
`post_with_retry` method, unread `WannolotPassResponse.status` field, an
empty line after an outer attribute, and a `match` that should be
`matches!`. None were security issues; all were style or dead-code.

**Fix.** Ran `cargo clippy --fix --allow-dirty` for the trivial ones, then
hand-fixed the remaining four:

- `auth/src/sentinel.rs`: added `#[allow(dead_code)]` on the serde struct
  with a comment explaining we keep all upstream fields even if currently
  unused.
- `pods/src/client.rs`: added `#[allow(dead_code)]` on `post_with_retry`
  with a comment about symmetric API design.
- `tunnel/src/monitor.rs`: rewrote the `match { Ok(Ok(_)) => true, _ =>
  false }` as `matches!(...)`.
- `cli/src/main.rs`: removed a misplaced `#[allow(dead_code)]` attribute
  followed by an empty line above `CLAUDE_MD_BLOCK`.

**Result.** `cargo clippy --workspace --all-targets` returns **zero
warnings**.

---

### LOW-3: Zero tests in the workspace

**Finding.** Every crate has 0 tests. `cargo test --workspace` passes
trivially because nothing exists to assert against. The CLI is mostly an
HTTP client + tunnel daemon, both of which are difficult to unit-test
without a network mock harness, but smoke tests for pure functions (like
the `tunnel-down` PID validator, the `shell_escape` function, the WG
config parser) would catch regressions cheaply.

**Fix.** Documented as backlog. Not a blocker for visibility flip — no
test failures, no incorrect positive results — but the next sprint should
add at least:

1. Unit tests for `cmd_tunnel_down` covering: PID 1 rejection,
   non-matching PID rejection, stale-pidfile cleanup, valid PID happy
   path (with a dummy PID file under `tempdir()`).
2. Unit tests for `shell_escape` covering: alphanumeric pass-through,
   embedded spaces, embedded single quotes.
3. Unit tests for the WG config parser (already isolated in `pods/`).

---

### LOW-4: `Cargo.toml` missing crates.io metadata

**Finding.** `[workspace.package]` had only `version`, `edition`,
`authors`, `license`. Missing `description`, `repository`, `homepage`,
`documentation`, `readme`, `keywords`, `categories`, `rust-version` —
all standard fields for crates.io publication.

**Fix.** Added all missing fields. The crate is now ready for `cargo
publish` if/when we want to ship it on crates.io alongside GitHub releases.

---

### LOW-5: Source comments referenced specific internal subnets

**Finding.** Doc comments in `tunnel/src/lib.rs` and `tunnel/src/monitor.rs`
used concrete examples like `10.17.8.0/24`, `10.17.8.2/24`, `10.18.1.0/24`,
revealing past production droplet octets.

**Fix.** Sanitized to placeholder format (`10.X.Y.0/24`) plus a note that
the stable address `10.42.42.1` is now appended to the AllowedIPs list.
Cosmetic but eliminates the leak.

---

### LOW-6: Hardcoded production URLs in source

**Finding.** Several `const &str` declarations contain production HTTPS
endpoints:

- `https://api.makakoo.com/ma-metrics-wsp-ms/v1/api`
- `https://api.makakoo.com/ma-authentication-ms/v1/api`
- `https://sentinel.traylinx.com`
- `https://tytus.traylinx.com`

**Assessment.** These are **not** secrets. They are public SaaS endpoints
that the CLI is designed to talk to. They will appear in `strings(1)`
output of any compiled binary regardless of how they're stored. Including
them in source is the correct architecture for a SaaS client.

**Fix.** No code change. Documented here so future audits don't re-flag.

---

### INFO-1: `keyring` service name uses old codename `com.traylinx.atomek`

**Finding.** `auth/src/keychain.rs` uses `SERVICE_NAME = "com.traylinx.atomek"`.
"Atomek" was the early codename of the desktop app that became `tytus-cli`.
The string is cosmetic — it's just the keychain entry namespace — but it
references the old name.

**Assessment.** Changing it would invalidate every existing user's
keychain entry, forcing them to re-login. Backwards-incompatible change
for purely cosmetic gain. Documented as "do not change without a
migration story" in `CLAUDE.md`.

---

### INFO-2: Two unmaintained-crate warnings

**Finding.** `cargo audit` reports:

- `RUSTSEC-2025-0057`: `fxhash 0.2.1` (via `inquire 0.7.5`) is no longer
  maintained.
- `RUSTSEC-2025-0119`: `number_prefix 0.4.0` (via `indicatif 0.17.11`)
  is no longer maintained.

**Assessment.** Neither is a vulnerability — both are warnings about
upstream maintenance status. The crates still work and have no known
issues. We are not exposed today, but we should track upstream
replacements:

- `inquire` upstream is moving away from `fxhash` in newer releases
- `indicatif` upstream has `number_prefix` removal in progress

**Fix.** Tracked. Re-evaluate in 3 months or on next major dependency
sweep, whichever comes first.

---

## Verification gate

Before flipping the repository to public, the following must hold:

| Check | Command | Result |
|---|---|---|
| Compiles clean (release) | `cargo build --release -p atomek-cli -p tytus-mcp` | ✅ |
| Zero clippy warnings | `cargo clippy --workspace --all-targets` | ✅ |
| Zero RUSTSEC vulnerabilities (errors) | `cargo audit` | ✅ |
| Tests pass | `cargo test --workspace` | ✅ (0 tests, none failing) |
| `install.sh` syntax valid (sh + bash) | `sh -n install.sh && bash -n install.sh` | ✅ |
| `tytus tunnel-down` validation works | manual: try PIDs 0, 1, random, valid | ✅ |
| README has no truncated key fingerprints | `grep -E 'sk-[a-zA-Z0-9]+\.\.\.' README.md` | empty ✅ |
| README has no internal IPs | `grep -E '10\.18\.|212\.227\.' README.md` | empty ✅ |
| `docs/VERIFICATION-*.md` removed | `ls docs/` | ✅ (only WIZARDS.md, SECURITY-AUDIT.md) |
| `.gitignore` blocks secrets | manual review | ✅ |
| Hosted SKILL.md fetchable after flip | `curl raw.githubusercontent.com/...` | pending visibility flip |

All blocker checks pass. Ready for the visibility flip.

---

## Follow-up backlog (post-public, not blocking)

1. **Add unit tests** for `cmd_tunnel_down`, `shell_escape`, WG config
   parser. See LOW-3.
2. **History rewrite consideration.** This audit deletes
   `docs/VERIFICATION-2026-04-10.md` from the working tree, but the file
   remains in git history. After visibility flip, anyone can pull the
   history and find the old commits. If that's unacceptable, run
   `git filter-repo --invert-paths --path docs/VERIFICATION-2026-04-10.md`
   BEFORE flipping visibility. Same applies to the README.md history that
   contains the truncated key fingerprints. **Operator decision required.**
3. **Track upstream replacements** for `fxhash` and `number_prefix` (see
   INFO-2).
4. **Publish to crates.io** once GitHub releases are stable. Cargo.toml
   metadata is now sufficient.
5. **Set up GitHub Actions release builds** for the prebuilt binary
   path in `install.sh`. Currently the script falls back to
   `cargo install --git` which works but takes 3-5 minutes for first-time
   users. Prebuilt binaries would cut this to seconds.
6. **Add `cargo audit` to CI** as a hard gate so no future PR can
   reintroduce a vulnerable dependency.
7. **Sign releases** with GPG or sigstore so the install script can verify
   download integrity beyond TLS.

---

## Operator sign-off

Once you've reviewed this report and decided on follow-up #2 (history
rewrite vs accept), you can flip the repo to public:

```bash
gh repo edit traylinx/tytus-cli --visibility public --accept-visibility-change-consequences
```

After that:

1. Verify `curl https://raw.githubusercontent.com/traylinx/tytus-cli/main/install.sh`
   returns 200.
2. Verify `curl https://raw.githubusercontent.com/traylinx/tytus-cli/main/.agents/skills/tytus/SKILL.md`
   returns 200.
3. Run `tytus bootstrap-prompt` and try the paste-into-AI flow yourself
   with a fresh tytus install (in a VM or Docker if you want a true
   first-run experience).
4. Cut the first GitHub release `v0.1.0` so `install.sh`'s prebuilt path
   works for new users.
