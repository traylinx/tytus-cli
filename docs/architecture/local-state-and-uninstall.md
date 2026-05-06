# ADR: Local state, data retention, and uninstall scope

**Status:** Accepted for Phase 0.  
**Date:** 2026-05-06.  
**Sprint:** TYTUS-OS-MULTIPLATFORM.

## Decision

Tytus uses file-based local state through the Phase 2 platform path substrate for this sprint. SQLite is deferred until evidence shows JSON/file state cannot support migrations, support bundles, and repair safely.

Secrets do not live in the state files. Long-lived tokens and credential material live in the OS credential store or the explicit secure fallback defined by the credential phase.

## State categories

| Category | Storage | Retention |
|---|---|---|
| Session/auth metadata | app config/state file plus secret refs | Preserved across repair and normal uninstall. |
| Refresh tokens/API keys | OS credential store | Removed only on logout or explicit data wipe. |
| Pod selection/allocation metadata | app data state file | Preserved across repair and normal uninstall. |
| Tunnel generated config | app data with secret refs redacted | Removed on explicit data wipe; repaired on tunnel repair. |
| Runtime IPC/session files | runtime dir | Removed on daemon stop/reboot/repair. |
| Logs | logs dir with rotation/redaction | Preserved on normal uninstall unless user chooses data wipe. |
| TytusOS cached web assets | cache/app data | Can be removed on repair/update/uninstall. |
| User-generated content | user data path | Never removed without explicit separate confirmation. |

## File format

Use versioned JSON for Phase 1/2 state files:

- `schema_version`
- `app_version`
- `updated_at`
- typed sections per feature

Every reader must tolerate older schema versions and produce repair-friendly errors for corrupt files.

## Uninstall behavior

Normal uninstall removes:

- binaries
- services/autostart hooks
- Tytus-owned firewall/helper rules
- runtime files
- update staging files

Normal uninstall preserves:

- user-generated content
- logs
- pod metadata
- credentials unless user explicitly chooses logout/data removal

Explicit data removal wipes:

- credentials/tokens
- pod metadata
- logs
- cached web assets
- tunnel configs

The UI/installer must distinguish uninstall from data wipe in plain language.

## Support bundle redaction

Support bundles may include state shape, versions, errors, and paths. They must redact:

- tokens
- API keys
- pod private keys
- IPC tokens
- update signing internals
- user-generated content unless explicitly attached by the user
