"""`tytus lope install / uninstall` — automate device pairing + lope config.

Install does two things, both idempotent:

1. **Device pre-pairing on the pod.** Reads `/app/workspace/.openclaw/devices/paired.json`
   via `tytus exec`, adds our SDK's Ed25519 device entry with full operator
   scopes, writes it back. Without this, the gateway's `clearUnboundScopes`
   path strips write scope from token-only connects and every `sessions.create`
   fails (observed behaviour, see docs/DESIGN-TYTUS-LOPE-TEAMMATES.md §6).

2. **Lope provider registration.** Adds a `subprocess` provider entry to
   `~/.lope/config.json` (merging, not clobbering) so `lope negotiate
   --validators tytus-openclaw-<pod>` Just Works.

Uninstall reverses both. Device removal is best-effort; if the pod is down we
still wipe the lope entry so lope stops routing to a dead teammate.
"""

from __future__ import annotations

import argparse
import base64
import json
import os
import subprocess
import sys
import time
from dataclasses import dataclass
from pathlib import Path

from tytus_sdk.identity import DeviceIdentity, load_or_create_identity


TYTUS_SDK_DISPLAY_NAME = "tytus-sdk"
TYTUS_SDK_CLIENT_ID = "gateway-client"
TYTUS_SDK_CLIENT_MODE = "backend"
TYTUS_SDK_PLATFORM = "linux"
PAIRED_JSON_PATH = "/app/workspace/.openclaw/devices/paired.json"
OPERATOR_SCOPES = ["operator.read", "operator.write", "operator.admin"]
LOPE_CONFIG = Path.home() / ".lope" / "config.json"


@dataclass
class InstallResult:
    pod_id: str
    agent_type: str
    device_id: str
    provider_name: str
    paired_on_pod: bool
    lope_registered: bool


def _run_tytus_exec(pod_id: str, shell_cmd: str, timeout: int = 20) -> str:
    """Execute a shell command inside the pod via the Tytus CLI. Returns stdout."""
    out = subprocess.run(
        ["tytus", "exec", "--pod", pod_id, "--timeout", str(timeout), shell_cmd],
        capture_output=True,
        text=True,
        timeout=timeout + 10,
    )
    if out.returncode != 0:
        raise RuntimeError(
            f"tytus exec (pod {pod_id}) failed: rc={out.returncode}\n"
            f"stderr: {out.stderr.strip()}"
        )
    # `tytus exec` prefixes every invocation with "Running on pod NN..." —
    # strip that and anything after the last `\n` of genuine output.
    lines = out.stdout.splitlines()
    return "\n".join(l for l in lines if not l.startswith("Running on pod"))


def _read_paired_json(pod_id: str) -> dict:
    """Fetch the pod's paired.json as a dict (empty if missing)."""
    raw = _run_tytus_exec(
        pod_id,
        f"cat {PAIRED_JSON_PATH} 2>/dev/null || echo '{{}}'",
        timeout=15,
    )
    raw = raw.strip()
    if not raw:
        return {}
    try:
        return json.loads(raw)
    except json.JSONDecodeError as e:
        raise RuntimeError(f"paired.json on pod {pod_id} is not valid JSON: {e}\nRaw:\n{raw[:500]}")


def _write_paired_json(pod_id: str, data: dict) -> None:
    """Write the dict back to the pod atomically via base64-in-heredoc.

    We avoid shell quoting pitfalls by base64-encoding the JSON and decoding
    on the pod side. `base64` is in busybox/coreutils on all our agent images.
    """
    encoded = base64.b64encode(json.dumps(data, indent=2).encode("utf-8")).decode("ascii")
    cmd = (
        f"echo '{encoded}' | base64 -d > {PAIRED_JSON_PATH}.tmp && "
        f"chmod 600 {PAIRED_JSON_PATH}.tmp && "
        f"mv {PAIRED_JSON_PATH}.tmp {PAIRED_JSON_PATH} && "
        f"echo OK"
    )
    result = _run_tytus_exec(pod_id, cmd, timeout=20)
    if "OK" not in result:
        raise RuntimeError(f"Failed to write paired.json on pod {pod_id}: {result!r}")


def _device_entry(identity: DeviceIdentity, display_name: str = TYTUS_SDK_DISPLAY_NAME) -> dict:
    ts = int(time.time() * 1000)
    return {
        "deviceId": identity.device_id,
        "publicKey": identity.public_key_b64url,
        "platform": TYTUS_SDK_PLATFORM,
        "clientId": TYTUS_SDK_CLIENT_ID,
        "clientMode": TYTUS_SDK_CLIENT_MODE,
        "role": "operator",
        "roles": ["operator"],
        "scopes": OPERATOR_SCOPES[:],
        "approvedScopes": OPERATOR_SCOPES[:],
        "tokens": {},
        "createdAtMs": ts,
        "approvedAtMs": ts,
        "displayName": display_name,
    }


def pair_device_on_pod(pod_id: str, identity: DeviceIdentity | None = None) -> str:
    """Idempotently add our device to the pod's paired.json.
    Returns the device_id we paired."""
    ident = identity or load_or_create_identity()
    paired = _read_paired_json(pod_id)
    # Drop any legacy empty-key row (old bug) + existing row for our id.
    paired.pop("", None)
    paired[ident.device_id] = _device_entry(ident)
    _write_paired_json(pod_id, paired)
    return ident.device_id


def unpair_device_on_pod(pod_id: str, device_id: str) -> bool:
    """Remove our device from the pod's paired.json. Returns True if present
    and removed; False if it wasn't there. Raises if pod unreachable."""
    paired = _read_paired_json(pod_id)
    if device_id not in paired:
        return False
    del paired[device_id]
    _write_paired_json(pod_id, paired)
    return True


# ── Lope config ───────────────────────────────────────────────────────────


def _sdk_entrypoint() -> list[str]:
    """The command lope will shell out to. We pin python3 explicitly so lope's
    environment doesn't pick a different interpreter."""
    return ["python3", "-m", "tytus_sdk", "lope_validate"]


def register_lope_provider(pod_id: str, agent_type: str = "openclaw") -> str:
    """Merge a subprocess provider entry into ~/.lope/config.json.

    Returns the provider name (e.g. 'tytus-openclaw-02').
    """
    LOPE_CONFIG.parent.mkdir(parents=True, exist_ok=True)
    cfg: dict = {}
    if LOPE_CONFIG.exists():
        try:
            cfg = json.loads(LOPE_CONFIG.read_text())
        except json.JSONDecodeError:
            cfg = {}
    cfg.setdefault("version", 1)
    cfg.setdefault("providers", [])
    cfg.setdefault("validators", [])

    provider_name = f"tytus-{agent_type}-{pod_id}"
    provider_entry = {
        "name": provider_name,
        "type": "subprocess",
        "command": _sdk_entrypoint() + ["--pod", pod_id, "--agent", agent_type, "{prompt}"],
    }
    # Replace-in-place; preserves list order for stable diffs.
    existing_idx = next(
        (i for i, p in enumerate(cfg["providers"]) if p.get("name") == provider_name),
        None,
    )
    if existing_idx is None:
        cfg["providers"].append(provider_entry)
    else:
        cfg["providers"][existing_idx] = provider_entry

    if provider_name not in cfg["validators"]:
        cfg["validators"].append(provider_name)

    LOPE_CONFIG.write_text(json.dumps(cfg, indent=2))
    return provider_name


def unregister_lope_provider(provider_name: str) -> bool:
    """Remove our entry from lope config. Returns True if something changed."""
    if not LOPE_CONFIG.exists():
        return False
    cfg = json.loads(LOPE_CONFIG.read_text())
    changed = False
    providers = cfg.get("providers", [])
    new_providers = [p for p in providers if p.get("name") != provider_name]
    if len(new_providers) != len(providers):
        cfg["providers"] = new_providers
        changed = True
    validators = cfg.get("validators", [])
    if provider_name in validators:
        cfg["validators"] = [v for v in validators if v != provider_name]
        changed = True
    if cfg.get("primary") == provider_name:
        cfg.pop("primary")
        changed = True
    if changed:
        LOPE_CONFIG.write_text(json.dumps(cfg, indent=2))
    return changed


# ── CLI entrypoints ───────────────────────────────────────────────────────


def cmd_install(args: argparse.Namespace) -> int:
    identity = load_or_create_identity()
    print(f"Using device identity: {identity.device_id}", file=sys.stderr)

    print(f"→ Pairing device on pod {args.pod}…", file=sys.stderr)
    pair_device_on_pod(args.pod, identity)
    print("  ✓ device paired (scopes: operator.read, operator.write, operator.admin)", file=sys.stderr)

    print(f"→ Registering lope provider in {LOPE_CONFIG}…", file=sys.stderr)
    provider_name = register_lope_provider(args.pod, args.agent)
    print(f"  ✓ provider '{provider_name}' registered as a lope validator", file=sys.stderr)

    result = InstallResult(
        pod_id=args.pod,
        agent_type=args.agent,
        device_id=identity.device_id,
        provider_name=provider_name,
        paired_on_pod=True,
        lope_registered=True,
    )
    if args.json:
        print(json.dumps(result.__dict__, indent=2))
    else:
        print(
            f"\nDone. Try: lope negotiate --validators {provider_name} \"your goal\"",
            file=sys.stderr,
        )
    return 0


def cmd_uninstall(args: argparse.Namespace) -> int:
    identity = load_or_create_identity()
    provider_name = f"tytus-{args.agent}-{args.pod}"

    paired = False
    try:
        paired = unpair_device_on_pod(args.pod, identity.device_id)
        print(
            f"{'✓ removed' if paired else '— not present'}: device on pod {args.pod}",
            file=sys.stderr,
        )
    except RuntimeError as e:
        print(f"⚠  pod {args.pod} unreachable — skipping device unpair ({e})", file=sys.stderr)

    unregistered = unregister_lope_provider(provider_name)
    print(
        f"{'✓ removed' if unregistered else '— not present'}: lope provider {provider_name}",
        file=sys.stderr,
    )

    if args.json:
        print(json.dumps({
            "pod_id": args.pod,
            "agent_type": args.agent,
            "device_id": identity.device_id,
            "provider_name": provider_name,
            "device_unpaired": paired,
            "lope_unregistered": unregistered,
        }, indent=2))
    return 0


def cmd_list(args: argparse.Namespace) -> int:
    """List installed tytus-* lope providers."""
    if not LOPE_CONFIG.exists():
        if args.json:
            print(json.dumps({"providers": []}))
        else:
            print("No lope config.", file=sys.stderr)
        return 0
    cfg = json.loads(LOPE_CONFIG.read_text())
    tytus_providers = [
        p for p in cfg.get("providers", [])
        if p.get("name", "").startswith("tytus-")
    ]
    if args.json:
        print(json.dumps({"providers": tytus_providers}, indent=2))
    else:
        if not tytus_providers:
            print("No Tytus providers registered in lope.", file=sys.stderr)
        else:
            for p in tytus_providers:
                active = p["name"] in cfg.get("validators", [])
                print(f"  {'✓' if active else '✗'} {p['name']}")
    return 0
