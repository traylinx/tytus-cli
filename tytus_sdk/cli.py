"""`python3 -m tytus_sdk <subcommand>` — debug/dev entry point.

This is the subprocess that `tytus lope ask` shells out to and that the
LopeBridge validator wraps. Keep it stable — it's the wire format between
Rust and Python.
"""

from __future__ import annotations

import argparse
import json
import logging
import subprocess
import sys

from tytus_sdk import __version__
from tytus_sdk.adapter import AgentMessage


def _discover_pod_port(pod_id: str) -> int:
    """Read `tytus status --json`, find the pod, derive its forwarder port.

    Forwarder ports follow the convention `18700 + pod_num` (e.g. pod 02 → 18702).
    """
    try:
        out = subprocess.check_output(["tytus", "status", "--json"], timeout=15)
    except (subprocess.CalledProcessError, FileNotFoundError, subprocess.TimeoutExpired) as e:
        raise RuntimeError(f"Couldn't run `tytus status --json`: {e}") from e

    data = json.loads(out)
    for pod in data.get("pods", []):
        if str(pod.get("pod_id")) == pod_id:
            break
    else:
        raise RuntimeError(f"Pod {pod_id!r} not found in `tytus status`")
    # Pod IDs are zero-padded strings like "02" — treat as decimal
    try:
        num = int(pod_id)
    except ValueError as e:
        raise RuntimeError(f"Pod id {pod_id!r} is not numeric") from e
    return 18700 + num


def cmd_ask(args: argparse.Namespace) -> int:
    from tytus_sdk.adapters.openclaw import OpenClawAdapter

    if args.agent != "openclaw":
        print(
            f"Error: agent {args.agent!r} is not implemented yet. Phase 1 only "
            f"supports openclaw; hermes + switchai land in phase 4.",
            file=sys.stderr,
        )
        return 2

    port = _discover_pod_port(args.pod)
    prompt = args.prompt
    if prompt == "-":
        prompt = sys.stdin.read()

    messages: list[AgentMessage] = []
    if args.system:
        messages.append(AgentMessage("system", args.system))
    messages.append(AgentMessage("user", prompt))

    adapter = OpenClawAdapter(pod_id=args.pod, forwarder_port=port)
    try:
        reply = adapter.ask(messages, timeout_s=args.timeout)
    finally:
        adapter.close()

    if args.json:
        print(json.dumps({
            "pod_id": args.pod,
            "agent": args.agent,
            "identity": adapter.identify().__dict__,
            "reply": reply,
        }, indent=2, default=str))
    else:
        print(reply)
    return 0


def cmd_identity(args: argparse.Namespace) -> int:
    from tytus_sdk.identity import default_identity_path, load_or_create_identity

    ident = load_or_create_identity()
    out = {
        "device_id": ident.device_id,
        "public_key": ident.public_key_b64url,
        "path": str(default_identity_path()),
    }
    print(json.dumps(out, indent=2))
    return 0


def build_parser() -> argparse.ArgumentParser:
    p = argparse.ArgumentParser(prog="tytus_sdk")
    p.add_argument("--version", action="version", version=f"tytus-sdk {__version__}")
    p.add_argument("-v", "--verbose", action="store_true")
    sub = p.add_subparsers(dest="cmd", required=True)

    ask = sub.add_parser("ask", help="Single-turn ask against a pod's agent")
    ask.add_argument("--agent", default="openclaw")
    ask.add_argument("--pod", required=True, help="Pod id (e.g. '02')")
    ask.add_argument("--system", help="Optional system prompt")
    ask.add_argument("--timeout", type=int, default=120)
    ask.add_argument("--json", action="store_true", help="Emit JSON with identity + reply")
    ask.add_argument("prompt", help="User prompt (use '-' to read from stdin)")
    ask.set_defaults(func=cmd_ask)

    ident = sub.add_parser("identity", help="Print the device identity (path + pubkey)")
    ident.set_defaults(func=cmd_identity)

    # ── install / uninstall / list — device pairing + lope registration ──
    from tytus_sdk.install import cmd_install, cmd_uninstall, cmd_list

    install = sub.add_parser("install", help="Pair device on pod + register lope provider")
    install.add_argument("--pod", required=True)
    install.add_argument("--agent", default="openclaw")
    install.add_argument("--json", action="store_true")
    install.set_defaults(func=cmd_install)

    uninstall = sub.add_parser("uninstall", help="Unpair device on pod + remove lope provider")
    uninstall.add_argument("--pod", required=True)
    uninstall.add_argument("--agent", default="openclaw")
    uninstall.add_argument("--json", action="store_true")
    uninstall.set_defaults(func=cmd_uninstall)

    lst = sub.add_parser("list", help="List Tytus providers registered in lope")
    lst.add_argument("--json", action="store_true")
    lst.set_defaults(func=cmd_list)

    # ── lope_validate — VERDICT-emitting validator subprocess ──
    from tytus_sdk.lope_bridge import cmd_lope_validate

    validate = sub.add_parser(
        "lope_validate",
        help="Lope validator subprocess (stdin → VERDICT block on stdout)",
    )
    validate.add_argument("--pod", required=True)
    validate.add_argument("--agent", default="openclaw")
    validate.add_argument("--timeout", type=int, default=300)
    validate.add_argument(
        "prompt", nargs="?", default="-",
        help="Review prompt (default: read from stdin)",
    )
    validate.set_defaults(func=cmd_lope_validate)

    # ── bridge — HarveyBridge daemon + test + rotate ──
    from tytus_sdk.bridge_daemon import (
        cmd_bridge_run, cmd_bridge_status, cmd_bridge_rotate_token, cmd_bridge_test,
    )

    bridge = sub.add_parser("bridge", help="HarveyBridge daemon (pod → brain reverse channel)")
    bridge_sub = bridge.add_subparsers(dest="bridge_cmd", required=True)

    b_run = bridge_sub.add_parser("run", help="Start HTTP listener + per-pod pollers")
    b_run.set_defaults(func=cmd_bridge_run)

    b_status = bridge_sub.add_parser("status", help="Health-check the running daemon")
    b_status.set_defaults(func=cmd_bridge_status)

    b_rotate = bridge_sub.add_parser("rotate-token", help="Rotate the shared bridge secret")
    b_rotate.add_argument("--json", action="store_true")
    b_rotate.set_defaults(func=cmd_bridge_rotate_token)

    b_test = bridge_sub.add_parser("test", help="Send a synthetic notify to the running daemon")
    b_test.add_argument("--pod", default="00")
    b_test.add_argument("--agent", default="openclaw")
    b_test.add_argument("--json", action="store_true")
    b_test.add_argument("message", default="test notify", nargs="?")
    b_test.set_defaults(func=cmd_bridge_test)

    return p


def main(argv: list[str] | None = None) -> int:
    parser = build_parser()
    args = parser.parse_args(argv)
    logging.basicConfig(
        level=logging.DEBUG if args.verbose else logging.WARNING,
        format="%(asctime)s %(levelname)s %(name)s: %(message)s",
    )
    return args.func(args)


if __name__ == "__main__":
    sys.exit(main())
