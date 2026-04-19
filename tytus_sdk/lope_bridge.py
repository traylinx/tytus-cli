"""LopeBridge — subprocess validator that lope shells out to.

Contract (from ~/.lope/lope/validators.py:168):
  stdin  : the review prompt (markdown)
  stdout : raw assistant output that MUST contain a block
           ---VERDICT---\n{JSON}\n---END---
           with fields: status (PASS|NEEDS_FIX|FAIL), confidence (0.0-1.0),
           rationale, required_fixes, nice_to_have.

We wrap the caller's prompt with a VERDICT-demanding system preamble,
call the OpenClaw adapter, and return the agent's reply unchanged — lope's
validators.py already knows how to fish the VERDICT block out.

If the agent replies without a VERDICT block (e.g. refused the rubric or
used tool-calls-only), we emit a defensive `INFRA_ERROR` verdict so lope
doesn't hang.
"""

from __future__ import annotations

import argparse
import json
import os
import re
import subprocess
import sys

from tytus_sdk.adapter import AgentMessage


VERDICT_SYSTEM_PROMPT = """\
You are reviewing a sprint plan or implementation for an engineering team.

Read the user's prompt (which is a structured review request). Evaluate it
against the criteria it lists. Produce ONE verdict block at the end of your
reply in this EXACT format:

```
---VERDICT---
{
  "status": "PASS" | "NEEDS_FIX" | "FAIL",
  "confidence": 0.0-1.0,
  "rationale": "one or two sentences on the decision",
  "required_fixes": ["concrete fix 1", "..."],
  "nice_to_have": ["optional improvement 1", "..."]
}
---END---
```

Rules:
- status=PASS means you actually verified it works. Low confidence (< 0.7)
  means you are NOT yet sure it works — always NEEDS_FIX or FAIL then.
- required_fixes must be actionable imperatives, not vague suggestions.
- Your free-form prose goes BEFORE the ---VERDICT--- block. Keep it short.
"""


VERDICT_BLOCK_RE = re.compile(r"---VERDICT---\s*(\{.*?\})\s*---END---", re.DOTALL)


def _fallback_verdict(rationale: str, raw_reply: str) -> str:
    """Emit a defensive VERDICT block when the agent didn't follow the rubric."""
    body = {
        "status": "NEEDS_FIX",
        "confidence": 0.3,
        "rationale": rationale,
        "required_fixes": [
            "Agent reply did not include a ---VERDICT--- block — rerun with "
            "stricter system prompt or inspect raw reply for detail.",
        ],
        "nice_to_have": [],
    }
    return (
        f"{raw_reply}\n\n---VERDICT---\n{json.dumps(body, indent=2)}\n---END---\n"
    )


def _validate_verdict_body(raw: str) -> tuple[bool, str]:
    """Returns (ok, message). Light schema check — lope does strict validation
    downstream; we just ensure the block parses."""
    match = VERDICT_BLOCK_RE.search(raw)
    if not match:
        return False, "no ---VERDICT---…---END--- block in reply"
    try:
        body = json.loads(match.group(1))
    except json.JSONDecodeError as e:
        return False, f"verdict block is not valid JSON: {e}"
    if body.get("status") not in {"PASS", "NEEDS_FIX", "FAIL"}:
        return False, f"invalid status: {body.get('status')!r}"
    try:
        conf = float(body.get("confidence", -1))
    except (TypeError, ValueError):
        return False, "confidence must be a number"
    if not (0.0 <= conf <= 1.0):
        return False, f"confidence out of range: {conf}"
    if not body.get("rationale"):
        return False, "rationale must be non-empty"
    return True, "ok"


def cmd_lope_validate(args: argparse.Namespace) -> int:
    from tytus_sdk.adapters.openclaw import OpenClawAdapter
    from tytus_sdk.cli import _discover_pod_port

    prompt = args.prompt
    if prompt == "-" or not prompt:
        prompt = sys.stdin.read()
    if not prompt.strip():
        print("Error: empty prompt on stdin/argv", file=sys.stderr)
        return 2

    if args.agent != "openclaw":
        print(
            f"Error: agent {args.agent!r} not wired in Phase 1. "
            f"Hermes + SwitchAI land in Phase 6.",
            file=sys.stderr,
        )
        return 2

    port = _discover_pod_port(args.pod)
    adapter = OpenClawAdapter(pod_id=args.pod, forwarder_port=port)
    try:
        reply = adapter.ask(
            [
                AgentMessage("system", VERDICT_SYSTEM_PROMPT),
                AgentMessage("user", prompt),
            ],
            timeout_s=args.timeout,
        )
    finally:
        adapter.close()

    ok, msg = _validate_verdict_body(reply)
    if not ok:
        reply = _fallback_verdict(msg, reply)

    sys.stdout.write(reply)
    if not reply.endswith("\n"):
        sys.stdout.write("\n")
    return 0
